//! Signatures and offsets for Steam SotFS 1.03, and the game calls this mod makes.
//!
//! Pointer chains and the ones marked DS2S-META come from DS2S-META `DS2REData.cs`. Chains are written
//! the DS2S-META way: each offset is added and then dereferenced. The rest were read from the exe.

use crate::module::{self, Module};
use crate::scan::{rip_relative, Pattern};
use crate::mem;

/// `SizeOfImage` of the only exe we support. Anything else and the mod switches itself off.
pub const EXPECTED_IMAGE_SIZE: usize = 0x1D7_6000;

/// `mov rax, [rip+disp32]` loading the GameManagerImp ("BaseA") global.
pub const AOB_BASE_A: &str = "48 8B 05 ? ? ? ? 48 8B 58 38 48 85 DB 74 ? F6";
/// `ApplySpEffect(sp_effect_ctrl*, effect_struct*, _, _, _, f32 = -1.0)` (DS2S-META, S103 entry).
pub const AOB_APPLY_SP_EFFECT: &str = "48 89 6C 24 F8 48 8D 64 24 F8 48 8D 2D 33 A7 0A 00";
/// `ItemGive(item_bag*, item_structs*, u32 count, 0)`, adding items with no on-screen window.
pub const AOB_ITEM_GIVE: &str = "48 89 5C 24 18 56 57 41 56 48 83 EC 30 45 8B F1 41";
/// `FmgLookup(fmg*, u32 id) -> *const u16`: finds a string in a loaded text file, null if it has none.
pub const AOB_FMG_LOOKUP: &str = "48 89 5C 24 08 48 89 7C 24 10 33 DB 44 8B DA 48 8B F9 48 85 C9 75 0D 33 C0 48 8B";
/// `mov rsi, [rip+disp32]` loading the text manager global, in the function that fetches a string by
/// category and id: its text file for category `c` is at `+0x1d8 + c * 0x20`.
pub const AOB_TEXT_MANAGER: &str = "48 8B 35 ? ? ? ? 8B EA 4C 63 F1 48 85 F6";
/// The title flow's first state, `FeSubStateTitleInitBranch`: a byte saying the opening has been shown
/// sends it to the title (state 0x17) instead of the warning and logos, as when returning from a game.
pub const AOB_TITLE_INIT_BRANCH: &str = "33 C0 38 05 ? ? ? ? C6 05 ? ? ? ? 01 0F 95 C0 89 41 10";
/// `FeSubStateTitleSteamNetworkCheck`'s enter function, which picks the next state through `+0x10`: 1
/// is the online login, 2 is "not connected to Steam", 3 is a frame rate failure.
pub const AOB_TITLE_NETWORK_CHECK: &str = "48 89 5C 24 08 57 48 83 EC 20 48 8B 05 ? ? ? ? 48 8B D9 48 8B B8 F0 22 00 00";
/// Where that check's result 2 gets its target state, as `mov dword [rax+8], 0x24`.
pub const AOB_TITLE_NOT_CONNECTED_TARGET: &str = "C7 40 08 24 00 00 00 C6 40 0C 01 C7 40 10 FF FF FF FF 48 89 28 C7 40 20 02 00 00 00";
/// `FeGroupInGameReturnTitleCheck`'s "yes": what confirming Quit Game in the System menu runs. It ignores
/// its `this`, closes the menu and asks the game manager to save and go to the title.
pub const AOB_QUIT_TO_TITLE: &str = "48 83 EC 28 48 8B 0D ? ? ? ? 48 8B 89 E0 22 00 00 E8 ? ? ? ? 48 8B 05 ? ? ? ? 33 D2";
/// `FeSubStateTitleShutdown`'s enter function, which loads a global and sets its closing byte.
pub const AOB_TITLE_SHUTDOWN: &str = "48 8B 05 ? ? ? ? C6 80 3A 01 00 00 01 C3 CC 48 83 EC 68 48 8D 4C 24 20";

/// Every signature the mod depends on, for `resolve` and the offline scanner.
pub const SIGNATURES: [(&str, &str); 10] = [
    ("BaseA", AOB_BASE_A),
    ("ApplySpEffect", AOB_APPLY_SP_EFFECT),
    ("ItemGive", AOB_ITEM_GIVE),
    ("FmgLookup", AOB_FMG_LOOKUP),
    ("TextManager", AOB_TEXT_MANAGER),
    ("TitleInitBranch", AOB_TITLE_INIT_BRANCH),
    ("TitleNetworkCheck", AOB_TITLE_NETWORK_CHECK),
    ("TitleNotConnectedTarget", AOB_TITLE_NOT_CONNECTED_TARGET),
    ("QuitToTitle", AOB_QUIT_TO_TITLE),
    ("TitleShutdown", AOB_TITLE_SHUTDOWN),
];

/// The check's result meaning "not connected to Steam", the game's own route to playing offline.
pub const NETWORK_CHECK_NOT_CONNECTED: u32 = 2;
pub const OFF_NETWORK_CHECK_RESULT: usize = 0x10;
/// The title menu. The game's offline route goes through two dialogs and then here.
pub const TITLE_STATE_TOP_MENU: u32 = 0x47;

/// BaseA children.
const OFF_PLAYER_CTRL: usize = 0xd0;
const OFF_EVENT_MANAGER: usize = 0x70;
/// PlayerCtrl child.
const OFF_SP_EFFECT_CTRL: usize = 0x3e0;
/// BaseA child, and the character name within it (DS2S-META `PlayerName`).
const OFF_PLAYER_MISC: usize = 0xa8;
const OFF_PLAYER_NAME: usize = 0x114;
/// `game_state` while the player is in control.
const GAME_STATE_PLAYABLE: u32 = 30;
/// BaseA -> 0xa8 -> 0x10 -> 0x10 is the player's item bag.
const CHAIN_ITEM_BAG: [usize; 3] = [0xa8, 0x10, 0x10];
/// PlayerCtrl -> 0xf8 -> 0xf0 is the position block.
const CHAIN_POSITION: [usize; 2] = [0xf8, 0xf0];
const POS_FIELDS: usize = 0x20;
/// EventManager fields. The last bonfire reads as nothing once back at the title.
const OFF_LAST_BONFIRE: usize = 0x16c;
const CHAIN_LOADING: [usize; 2] = [0x80, 0x8];
const OFF_LOADING: usize = 0xbb4;
const OFF_GAME_STATE: usize = 0x24ac;
/// BaseA child holding the ItemParam resource (DS2S-META `ParamOffsets.txt`: A8;20;D8), and within
/// it, the pointer to the param, laid out the same as the file.
const CHAIN_ITEM_PARAM_RES: [usize; 2] = [0xa8, 0x20];
const OFF_PARAM_DATA: usize = 0xd8;
/// Byte saying the resource owns that data and must free it. A pointer from elsewhere trips an
/// assertion in the game's allocator when the game closes (`+0xaf6950` skips the free if this is 0).
const OFF_PARAM_DATA_OWNED: usize = 0xe8;
/// Within the text manager: ItemName, SimpleExplanation and DetailedExplanation, categories 8 to 10.
const OFF_TEXT_ITEM_NAME: usize = 0x2d8;
const OFF_TEXT_SIMPLE: usize = 0x2f8;
const OFF_TEXT_DETAILED: usize = 0x318;
/// The closing byte: the main loop starts shutting down once it's set.
const OFF_CLOSING: usize = 0x13a;

#[derive(Debug, Clone, Copy)]
pub struct Addresses {
    pub module: Module,
    /// Address of the BaseA global (dereference it to get BaseA).
    pub base_a_global: usize,
    pub apply_sp_effect: usize,
    pub item_give: usize,
    pub fmg_lookup: usize,
    /// Address of the text manager global (dereference it to get the manager).
    pub text_manager_global: usize,
    pub title_init_branch: usize,
    /// The byte saying the opening has been shown.
    pub opening_shown: usize,
    pub title_network_check: usize,
    pub title_not_connected_target: usize,
    pub quit_to_title: usize,
    /// Address of the global holding the closing byte's owner.
    pub closing_global: usize,
}

/// Which of the item's three strings a text file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemText {
    Name,
    Simple,
    Detailed,
}

#[derive(Debug)]
pub enum ResolveError {
    NoModule,
    WrongVersion { size: usize, timestamp: u32 },
    Pattern { name: &'static str, hits: usize },
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::NoModule => write!(f, "couldn't read the main module"),
            ResolveError::WrongVersion { size, timestamp } => write!(
                f,
                "unsupported DarkSoulsII.exe (SizeOfImage {size:#x}, timestamp {timestamp:#x}); expected {EXPECTED_IMAGE_SIZE:#x} (Steam 1.03)"
            ),
            ResolveError::Pattern { name, hits } => write!(f, "signature {name} matched {hits} times, expected 1"),
        }
    }
}

pub fn resolve() -> Result<Addresses, ResolveError> {
    let module = module::main_module().ok_or(ResolveError::NoModule)?;
    if module.size != EXPECTED_IMAGE_SIZE {
        return Err(ResolveError::WrongVersion { size: module.size, timestamp: module.timestamp });
    }
    let find = |name: &'static str, aob: &str| {
        Pattern::parse(aob).find_unique(&module).map_err(|hits| ResolveError::Pattern { name, hits })
    };
    let target = |name: &'static str, hit: usize, disp: usize, len: usize| {
        rip_relative(hit, disp, len).ok_or(ResolveError::Pattern { name, hits: 0 })
    };
    let title_init_branch = find("TitleInitBranch", AOB_TITLE_INIT_BRANCH)?;
    Ok(Addresses {
        module,
        base_a_global: target("BaseA", find("BaseA", AOB_BASE_A)?, 3, 7)?,
        apply_sp_effect: find("ApplySpEffect", AOB_APPLY_SP_EFFECT)?,
        item_give: find("ItemGive", AOB_ITEM_GIVE)?,
        fmg_lookup: find("FmgLookup", AOB_FMG_LOOKUP)?,
        text_manager_global: target("TextManager", find("TextManager", AOB_TEXT_MANAGER)?, 3, 7)?,
        title_init_branch,
        opening_shown: target("TitleInitBranch", title_init_branch, 4, 8)?,
        title_network_check: find("TitleNetworkCheck", AOB_TITLE_NETWORK_CHECK)?,
        title_not_connected_target: find("TitleNotConnectedTarget", AOB_TITLE_NOT_CONNECTED_TARGET)? + 3,
        quit_to_title: find("QuitToTitle", AOB_QUIT_TO_TITLE)?,
        closing_global: target("TitleShutdown", find("TitleShutdown", AOB_TITLE_SHUTDOWN)?, 3, 7)?,
    })
}

impl Addresses {
    fn base_a(&self) -> Option<usize> {
        mem::ptr(self.base_a_global)
    }

    fn player_ctrl(&self) -> Option<usize> {
        mem::ptr(self.base_a()? + OFF_PLAYER_CTRL)
    }

    /// The player is in the world and in control.
    pub fn playable(&self) -> bool {
        let Some(base) = self.base_a() else { return false };
        let state = mem::read::<u32>(base + OFF_GAME_STATE);
        let loading = mem::chain(base, &CHAIN_LOADING).and_then(|l| mem::read::<u8>(l + OFF_LOADING));
        let pos = self
            .player_ctrl()
            .and_then(|pc| mem::chain(pc, &CHAIN_POSITION))
            .and_then(|p| mem::read::<[f32; 3]>(p + POS_FIELDS));
        state == Some(GAME_STATE_PLAYABLE) && loading == Some(0) && pos.is_some()
    }

    /// Back at the title screen: the game is running, with no event manager data for a world.
    pub fn at_title(&self) -> bool {
        let Some(base) = self.base_a() else { return false };
        let last_bonfire = mem::ptr(base + OFF_EVENT_MANAGER).and_then(|em| mem::read::<u16>(em + OFF_LAST_BONFIRE));
        mem::read::<u32>(base + OFF_GAME_STATE).is_some() && last_bonfire.is_none()
    }

    pub fn character_name(&self) -> Option<Vec<u16>> {
        let bytes = mem::read_bytes(mem::ptr(self.base_a()? + OFF_PLAYER_MISC)? + OFF_PLAYER_NAME, 32)?;
        let units: Vec<u16> =
            bytes.as_chunks::<2>().0.iter().map(|b| u16::from_le_bytes(*b)).take_while(|&u| u != 0).collect();
        (!units.is_empty()).then_some(units)
    }

    pub fn sp_effect_ctrl(&self) -> Option<usize> {
        mem::ptr(self.player_ctrl()? + OFF_SP_EFFECT_CTRL)
    }

    /// Adds `amount` of an item to the inventory, silently. Returns false if the bag isn't available.
    ///
    /// # Safety
    /// Calls into the game. The player must be in control, and `id` must be a row in ItemParam.
    pub unsafe fn give_item(&self, id: u32, amount: u16) -> bool {
        #[repr(C, align(16))]
        struct ItemStruct {
            kind: i32,
            id: u32,
            durability: f32,
            amount: u16,
            upgrade: u8,
            infusion: u8,
        }
        type Give = unsafe extern "system" fn(usize, *const ItemStruct, u32, u32) -> u64;

        let Some(bag) = self.base_a().and_then(|b| mem::chain(b, &CHAIN_ITEM_BAG)) else { return false };
        let item = ItemStruct { kind: 3, id, durability: f32::MAX, amount, upgrade: 0, infusion: 0 };
        let give: Give = std::mem::transmute(self.item_give);
        give(bag, &item, 1, 0);
        true
    }

    /// Saves and goes to the title, exactly as confirming Quit Game in the System menu does.
    ///
    /// # Safety
    /// Calls into the game. The player must be in control.
    pub unsafe fn quit_to_title(&self) {
        type Quit = unsafe extern "system" fn(usize);
        let quit: Quit = std::mem::transmute(self.quit_to_title);
        quit(0);
    }

    /// Tells the game to close itself, as the title flow's own shutdown state does.
    pub fn close_game(&self) -> bool {
        mem::ptr(self.closing_global).is_some_and(|owner| mem::write::<u8>(owner + OFF_CLOSING, 1))
    }

    /// The resource holding ItemParam, once the game has loaded it.
    fn item_param_res(&self) -> Option<usize> {
        self.base_a().and_then(|b| mem::chain(b, &CHAIN_ITEM_PARAM_RES))
    }

    /// The ItemParam the game is using right now.
    pub fn item_param(&self) -> Option<usize> {
        mem::ptr(self.item_param_res()? + OFF_PARAM_DATA)
    }

    /// Points the game at `data` as its ItemParam. The game won't try to free it, or the old copy.
    pub fn set_item_param(&self, data: usize) -> bool {
        let Some(res) = self.item_param_res() else { return false };
        mem::write::<u8>(res + OFF_PARAM_DATA_OWNED, 0) && mem::write(res + OFF_PARAM_DATA, data)
    }

    /// Which item string a loaded text file is for, if any.
    pub fn item_text_kind(&self, fmg: usize) -> Option<ItemText> {
        let manager = mem::ptr(self.text_manager_global)?;
        [(OFF_TEXT_ITEM_NAME, ItemText::Name), (OFF_TEXT_SIMPLE, ItemText::Simple), (OFF_TEXT_DETAILED, ItemText::Detailed)]
            .into_iter()
            .find(|&(off, _)| mem::ptr(manager + off) == Some(fmg))
            .map(|(_, kind)| kind)
    }
}
