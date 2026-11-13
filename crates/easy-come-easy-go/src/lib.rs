//! Easy Come, Easy Go: starts Dark Souls II straight at the title menu, offline, and adds an item that
//! saves and quits to the desktop.
//!
//! The item and its text are added in memory, so no game files change. The quick start and the item
//! are found and switched on separately, so one failing leaves the other working, and
//! `easy_come_easy_go.ini` can turn each part off.

#![cfg(all(windows, target_pointer_width = "64"))]

use ds2::game::{self, Addresses, ItemText, QuickStart};
use ds2::settings::{self, Setting, Settings};
use ds2::{item, log, main_thread};
use minhook::MinHook;
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const DLL_PROCESS_ATTACH: u32 = 1;
const TICK: Duration = Duration::from_millis(50);
/// Saving and loading the title can be slow; this only gives up on closing, never the game.
const QUIT_TIMEOUT: Duration = Duration::from_secs(180);
/// How long the game must have been at the title before closing, so the world has finished unloading.
const TITLE_SETTLE: Duration = Duration::from_secs(2);
/// How long a game call may wait for the main thread. It polls every frame, so this is generous.
const MAIN_THREAD_WAIT: Duration = Duration::from_secs(2);

const TITLE: &str = "Easy Come, Easy Go";
const SETTINGS: [Setting; 4] = [
    Setting { name: "skip_logos", default: true, help: "Skip the warning screen and logos at startup." },
    Setting {
        name: "skip_network_check",
        default: true,
        help: "Skip the network check and go straight to the title menu, offline. Ignored under Seamless Co-op, which needs the check.",
    },
    Setting {
        name: "block_network",
        default: true,
        help: "Block all network access, so a modded game can't go online. Ignored under Seamless Co-op, which needs the network.",
    },
    Setting { name: "quit_item", default: true, help: "Give each character the Easy Go item, which saves and quits to the desktop." },
];

static ADDRS: OnceLock<Addresses> = OnceLock::new();
static QUICK: OnceLock<QuickStart> = OnceLock::new();
static CONFIG: OnceLock<Settings> = OnceLock::new();
static ORIG_APPLY_SP_EFFECT: AtomicUsize = AtomicUsize::new(0);
static ORIG_FMG_LOOKUP: AtomicUsize = AtomicUsize::new(0);
static ORIG_TITLE_INIT_BRANCH: AtomicUsize = AtomicUsize::new(0);
static ORIG_TITLE_NETWORK_CHECK: AtomicUsize = AtomicUsize::new(0);
static ORIG_WSA_STARTUP: AtomicUsize = AtomicUsize::new(0);
/// What `DllMain` saw: why the game counts as running under Seamless Co-op, if it does.
static SEAMLESS_AT_START: OnceLock<Option<&'static str>> = OnceLock::new();

/// The hooks only raise these; the mod's logic runs on its own thread.
static ITEM_USED: AtomicBool = AtomicBool::new(false);
static NETWORK_BLOCKED: AtomicBool = AtomicBool::new(false);
static NETWORK_REFUSED: AtomicBool = AtomicBool::new(false);
static NETWORK_LET_THROUGH: AtomicBool = AtomicBool::new(false);

/// The item's strings as the game wants them, NUL-terminated UTF-16 that lives as long as the process.
static ITEM_TEXT: OnceLock<[Vec<u16>; 3]> = OnceLock::new();

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn dll_path() -> Option<PathBuf> {
    ds2::module::module_path_of(DllMain as *const () as usize)
}

fn config() -> &'static Settings {
    CONFIG.get_or_init(|| match dll_path() {
        Some(dll) => settings::load(&settings::path_for(&dll), &SETTINGS),
        None => Settings::parse("", &SETTINGS),
    })
}

#[no_mangle]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        // The network hook can't wait for the thread: the game may start networking as soon as it
        // runs. Reading the settings file first is a plain file read. Everything else stays out of
        // the loader lock, and starts once DllMain returns. Under Seamless Co-op nothing is blocked.
        let seamless = ds2::seamless::detect();
        let _ = SEAMLESS_AT_START.set(seamless);
        if config().get("block_network") && seamless.is_none() {
            NETWORK_BLOCKED.store(unsafe { block_network() }, Ordering::SeqCst);
        }
        std::thread::spawn(run);
    }
    1
}

/// Keeps the game offline by making Windows sockets refuse to start, as Mod Engine does, so nothing
/// modded can reach the servers even if this mod's other changes don't apply.
unsafe fn block_network() -> bool {
    MinHook::create_hook_api("ws2_32", "WSAStartup", wsa_startup_detour as *mut c_void)
        .map(|orig| ORIG_WSA_STARTUP.store(orig as usize, Ordering::SeqCst))
        .and_then(|_| MinHook::enable_all_hooks())
        .is_ok()
}

const WSASYSNOTREADY: i32 = 10091;

type WsaStartupFn = unsafe extern "system" fn(u16, *mut c_void) -> i32;

/// Refuses, unless Seamless Co-op's DLL has arrived since startup: Steam's networking closes the game
/// when this fails, and Seamless keeps the game off FromSoftware's servers itself.
unsafe extern "system" fn wsa_startup_detour(version: u16, data: *mut c_void) -> i32 {
    let orig = ORIG_WSA_STARTUP.load(Ordering::SeqCst);
    if orig != 0 && ds2::seamless::dll_loaded() {
        NETWORK_LET_THROUGH.store(true, Ordering::SeqCst);
        let orig: WsaStartupFn = std::mem::transmute(orig);
        return orig(version, data);
    }
    NETWORK_REFUSED.store(true, Ordering::SeqCst);
    WSASYSNOTREADY
}

/// Why the game counts as running under Seamless Co-op: what `DllMain` saw, or what's true now.
fn seamless() -> Option<&'static str> {
    SEAMLESS_AT_START.get().copied().flatten().or_else(ds2::seamless::detect)
}

/// Logs what the network hook did since the last call.
fn log_network() {
    if NETWORK_REFUSED.swap(false, Ordering::SeqCst) {
        log::line("network start refused".into());
    }
    if NETWORK_LET_THROUGH.swap(false, Ordering::SeqCst) {
        log::line("network start let through: Seamless Co-op's DLL is loaded".into());
    }
}

fn run() {
    let dll = DllMain as *const () as usize;
    log::init(dll, "easy_come_easy_go.log");
    let config = config();
    log::line(format!("settings: {}", config.summary(&SETTINGS)));
    for problem in &config.problems {
        log::line(format!("settings file: {problem}, using the default"));
    }
    if let Some(dll) = dll_path() {
        match settings::write_defaults_if_missing(&settings::path_for(&dll), TITLE, &SETTINGS) {
            Ok(true) => log::line("wrote the settings file with the defaults".into()),
            Ok(false) => {}
            Err(e) => log::line(format!("couldn't write the settings file: {e}")),
        }
    }
    let seamless = seamless();
    if let Some(why) = seamless {
        log::line(format!("Seamless Co-op ({why}): network check and network left alone"));
    } else if config.get("block_network") {
        log::line(format!("network blocked: {}", NETWORK_BLOCKED.load(Ordering::SeqCst)));
    }
    let module = match game::game_module() {
        Ok(m) => m,
        Err(e) => {
            log::line(format!("disabled: {e}"));
            return;
        }
    };
    start_quick_start(&module, config, seamless.is_some());
    if !config.get("quit_item") {
        log::line("quit item turned off in the settings".into());
        watch_network_only();
    }

    let addrs = match game::resolve(&module) {
        Ok(a) => a,
        Err(e) => {
            log::line(format!("quit item disabled: {e}"));
            watch_network_only();
        }
    };
    let _ = ADDRS.set(addrs);
    let _ = ITEM_TEXT.set([utf16(item::NAME), utf16(item::SIMPLE), utf16(item::DETAILED)]);
    let hooked = ds2::hooks::serialized(|| unsafe {
        install(addrs.apply_sp_effect, apply_sp_effect_detour as *mut c_void, &ORIG_APPLY_SP_EFFECT)
            .and_then(|_| install(addrs.fmg_lookup, fmg_lookup_detour as *mut c_void, &ORIG_FMG_LOOKUP))
            .and_then(|_| MinHook::enable_all_hooks().map_err(|e| format!("enable: {e:?}")))
            .and_then(|_| main_thread::install(&module))
    });
    if let Err(e) = hooked {
        log::line(format!("quit item disabled, hooks failed: {e}"));
        watch_network_only();
    }
    log::line("ready".into());

    let dir = dll_path()
        .and_then(|p| p.parent().map(|d| d.join("easy_come_easy_go")))
        .unwrap_or_else(|| PathBuf::from("easy_come_easy_go"));
    Mod::new(dir).run(&addrs);
}

/// With no item to look after, the thread only reports refused network starts.
fn watch_network_only() -> ! {
    loop {
        std::thread::sleep(TICK);
        log_network();
    }
}

/// The parts of the quick start the settings ask for. Seamless Co-op needs the network check, so it
/// runs there whatever the settings say. A failure here is logged and leaves the item alone.
fn start_quick_start(module: &ds2::module::Module, config: &Settings, seamless: bool) {
    let (logos, check) = (config.get("skip_logos"), config.get("skip_network_check") && !seamless);
    if !logos && !check {
        return;
    }
    let quick = match game::resolve_quick_start(module) {
        Ok(q) => q,
        Err(e) => {
            log::line(format!("quick start disabled: {e}"));
            return;
        }
    };
    let _ = QUICK.set(quick);
    let hooked = ds2::hooks::serialized(|| unsafe {
        let mut result = Ok(());
        if logos {
            result = install(quick.title_init_branch, title_init_branch_detour as *mut c_void, &ORIG_TITLE_INIT_BRANCH);
        }
        if check && result.is_ok() {
            result = install(quick.title_network_check, title_network_check_detour as *mut c_void, &ORIG_TITLE_NETWORK_CHECK);
        }
        result.and_then(|_| MinHook::enable_all_hooks().map_err(|e| format!("enable: {e:?}")))
    });
    match hooked {
        Err(e) => log::line(format!("quick start disabled, hooks failed: {e}")),
        Ok(()) if check => {
            // Straight to the title menu from the check, past the "not connected" and "offline mode" dialogs.
            let direct = ds2::mem::write(quick.title_not_connected_target, game::TITLE_STATE_TOP_MENU);
            log::line(format!("quick start: skip logos {logos}, title shortcut {direct}"));
        }
        Ok(()) => log::line("quick start: skip logos true, network check left alone".into()),
    }
}

unsafe fn install(target: usize, detour: *mut c_void, orig: &AtomicUsize) -> Result<(), String> {
    let o = MinHook::create_hook(target as *mut c_void, detour).map_err(|e| format!("create {target:#x}: {e:?}"))?;
    orig.store(o as usize, Ordering::SeqCst);
    Ok(())
}

/// Reads a whole param from memory. It starts with its own length, as the file does.
fn read_param(addr: usize) -> Option<Vec<u8>> {
    let len = ds2::mem::read::<u32>(addr)? as usize;
    (0x40..=16 << 20).contains(&len).then(|| ds2::mem::read_bytes(addr, len)).flatten()
}

type ApplySpEffectFn = unsafe extern "system" fn(usize, *const u8, u64, u64, u64, u64, u64, u64) -> u64;

/// `ApplySpEffect(ctrl, effect_struct, ...)`. The full argument list isn't known, so the stack slots
/// pass through raw. Enemies use this too, so the effect id is checked first, and the pointer reads
/// behind the player check only happen for the item's id.
unsafe extern "system" fn apply_sp_effect_detour(
    ctrl: usize,
    effect: *const u8,
    r8: u64,
    r9: u64,
    s5: u64,
    s6: u64,
    s7: u64,
    s8: u64,
) -> u64 {
    let orig: ApplySpEffectFn = std::mem::transmute(ORIG_APPLY_SP_EFFECT.load(Ordering::SeqCst));
    let ret = orig(ctrl, effect, r8, r9, s5, s6, s7, s8);
    let id = ds2::mem::read::<i32>(effect as usize).unwrap_or(0);
    if id == item::ITEM as i32 && ADDRS.get().and_then(|a| a.sp_effect_ctrl()) == Some(ctrl) {
        ITEM_USED.store(true, Ordering::SeqCst);
    }
    ret
}

type TitleStateFn = unsafe extern "system" fn(usize);

/// Skips the warning and logos, by saying they've been shown before the game asks.
unsafe extern "system" fn title_init_branch_detour(state: usize) {
    if let Some(quick) = QUICK.get() {
        ds2::mem::write::<u8>(quick.opening_shown, 1);
    }
    let orig: TitleStateFn = std::mem::transmute(ORIG_TITLE_INIT_BRANCH.load(Ordering::SeqCst));
    orig(state);
}

/// Answers the network check with "not connected", so the game never tries to log in. The original
/// isn't called, since choosing the route is all it does.
unsafe extern "system" fn title_network_check_detour(state: usize) {
    ds2::mem::write(state + game::OFF_NETWORK_CHECK_RESULT, game::NETWORK_CHECK_NOT_CONNECTED);
}

type FmgLookupFn = unsafe extern "system" fn(usize, u32) -> *const u16;

/// Supplies the item's text in whichever language is loaded, so no text files need changing. Called
/// constantly, so anything but the item's id passes straight through.
unsafe extern "system" fn fmg_lookup_detour(fmg: usize, id: u32) -> *const u16 {
    let orig: FmgLookupFn = std::mem::transmute(ORIG_FMG_LOOKUP.load(Ordering::SeqCst));
    let found = orig(fmg, id);
    if id != item::ITEM || !found.is_null() {
        return found;
    }
    let index = match ADDRS.get().and_then(|a| a.item_text_kind(fmg)) {
        Some(ItemText::Name) => 0,
        Some(ItemText::Simple) => 1,
        Some(ItemText::Detailed) => 2,
        None => return found,
    };
    ITEM_TEXT.get().map_or(found, |text| text[index].as_ptr())
}

/// Where the item row stands in the param the game is using right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemRow {
    /// Nothing checked yet, or the game hasn't loaded ItemParam.
    Unknown,
    /// This param has the row, so the item is safe to give.
    In(usize),
    /// This param can't take the row. Giving the item could crash the game, so it isn't given.
    Failed(usize),
}

struct Mod {
    dir: PathBuf,
    item_row: ItemRow,
    character: Option<Vec<u16>>,
    item_settled: bool,
    main_thread_logged: bool,
    /// The item was used: the game is saving and heading to the title, then it closes.
    quitting: Option<Instant>,
    at_title_since: Option<Instant>,
}

impl Mod {
    fn new(dir: PathBuf) -> Mod {
        Mod {
            dir,
            item_row: ItemRow::Unknown,
            character: None,
            item_settled: false,
            main_thread_logged: false,
            quitting: None,
            at_title_since: None,
        }
    }

    fn run(mut self, addrs: &Addresses) -> ! {
        loop {
            std::thread::sleep(TICK);
            self.tick(addrs);
        }
    }

    fn tick(&mut self, addrs: &Addresses) {
        main_thread::find_main_thread();
        if !self.main_thread_logged && main_thread::is_running() {
            self.main_thread_logged = true;
            log::line("game calls run on the main thread".into());
        }
        log_network();
        self.keep_item_row(addrs);
        if addrs.playable() {
            self.note_character(addrs);
            self.grant_item(addrs);
        }
        if ITEM_USED.swap(false, Ordering::SeqCst) {
            self.on_item_used(addrs);
        }
        self.finish_quit(addrs);
    }

    /// Points the game at a copy of ItemParam with the item's row added, whenever the param it's using
    /// doesn't have one. The old copy stays in place, since other code may still be reading it.
    fn keep_item_row(&mut self, addrs: &Addresses) {
        let Some(current) = addrs.item_param() else { return };
        if matches!(self.item_row, ItemRow::In(p) | ItemRow::Failed(p) if p == current) {
            return;
        }
        self.item_row = match read_param(current).map(|bytes| item::add_row(&bytes)) {
            Some(Ok(Some(with_row))) => {
                let copy: &'static mut [u8] = Vec::leak(with_row);
                let new = copy.as_ptr() as usize;
                if addrs.set_item_param(new) {
                    log::line(format!("item row added: ItemParam {current:#x} -> {new:#x}, {} bytes", copy.len()));
                    ItemRow::In(new)
                } else {
                    log::line("couldn't update ItemParam's pointer".into());
                    ItemRow::Failed(current)
                }
            }
            Some(Ok(None)) => ItemRow::In(current),
            Some(Err(e)) => {
                log::line(format!("couldn't add the item row, so the item won't be given: {e}"));
                ItemRow::Failed(current)
            }
            None => {
                log::line(format!("couldn't read ItemParam at {current:#x}, so the item won't be given"));
                ItemRow::Failed(current)
            }
        };
    }

    fn note_character(&mut self, addrs: &Addresses) {
        let Some(name) = addrs.character_name() else { return };
        if self.character.as_ref() != Some(&name) {
            log::line(format!("character {}", String::from_utf16_lossy(&name)));
            self.character = Some(name);
            self.item_settled = false;
        }
    }

    /// Names can hold characters a file name can't, so the marker is named by the name's hex.
    fn marker(&self, character: &[u16]) -> PathBuf {
        let name: String = character.iter().map(|unit| format!("{unit:04x}")).collect();
        self.dir.join(format!("{name}.item"))
    }

    /// Gives each character one item, the first time it's seen in control. If the bag isn't available
    /// yet, this tries again on the next tick.
    fn grant_item(&mut self, addrs: &Addresses) {
        if self.item_settled || !matches!(self.item_row, ItemRow::In(_)) {
            return;
        }
        let Some(name) = self.character.clone() else { return };
        let marker = self.marker(&name);
        if marker.exists() {
            self.item_settled = true;
            return;
        }
        let a = *addrs;
        let given = main_thread::call("give the item", MAIN_THREAD_WAIT, move || unsafe { a.give_item(item::ITEM, 1) });
        if given == Some(true) {
            self.item_settled = true;
            log::line(format!("gave the item to {}", String::from_utf16_lossy(&name)));
            let written = std::fs::create_dir_all(&self.dir).and_then(|_| std::fs::write(&marker, ""));
            if let Err(e) = written {
                log::line(format!("couldn't record the item, it will be given again next launch: {e}"));
            }
        }
    }

    fn on_item_used(&mut self, addrs: &Addresses) {
        if self.quitting.is_some() {
            log::line("item use ignored: already quitting".into());
            return;
        }
        if !addrs.playable() {
            log::line("item used, but the player isn't in control".into());
            return;
        }
        let a = *addrs;
        let started = main_thread::call("quit", MAIN_THREAD_WAIT, move || {
            if !a.playable() {
                return false;
            }
            unsafe { a.quit_to_title() };
            true
        });
        if started == Some(true) {
            log::line("item used: saving and going to the title".into());
            self.quitting = Some(Instant::now());
        } else {
            log::line("item used, but the quit didn't start".into());
        }
    }

    /// Once at the title the game has saved, so it can close the same way its own shutdown does.
    fn finish_quit(&mut self, addrs: &Addresses) {
        let Some(since) = self.quitting else { return };
        if !addrs.at_title() {
            self.at_title_since = None;
        } else if self.at_title_since.get_or_insert_with(Instant::now).elapsed() >= TITLE_SETTLE {
            self.quitting = None;
            let closing = addrs.close_game();
            log::line(format!("at the title: closing={closing}"));
            return;
        }
        if since.elapsed() > QUIT_TIMEOUT {
            self.quitting = None;
            log::line("gave up on closing, the title never came".into());
        }
    }
}
