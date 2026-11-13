# Easy Come, Easy Go

A mod for Dark Souls II: Scholar of the First Sin that gets you in and out of the game quickly.

- **Easy come:** the game skips the logos and the network check, and goes straight to the title menu.
- **Easy go:** a new reusable item with a familiar, unpleasant icon. Use it to save your game and quit to
  the desktop, without going through the System menu and the title screen.

Every character gets one Easy Go in its inventory the first time you load it after installing. Put it in
an item slot to use it.

**This mod keeps the game offline** by default. While it's installed, the game can't connect to its
servers, so there's no multiplayer, messages or summoning. Seamless Co-op players can turn that off: see
Settings.

## Install

You need the Steam version of Scholar of the First Sin, 1.03. On any other exe, the mod logs that and
turns off everything except the network block.

1. Back up your saves, from `%APPDATA%\DarkSoulsII`.
2. If `Game\dinput8.dll` already exists and came from a mod other than Lazy Loader, rename it to something
   like `othermod.dll` and move it to `Game\dllMods\`, creating the folder if needed.
3. Extract the zip into the game's Steam folder, the one that contains `Game\`. Vortex's DS2 support
   deploys to this folder too.

That's all. The zip includes [Lazy Loader](https://www.nexusmods.com/darksouls3/mods/677), which loads
the mod. If you already use it, your `lazyLoad.ini` isn't touched.

## Settings

The first launch writes `Game\dllMods\easy_come_easy_go.ini`. Each line is a setting, `true` or `false`:

| Setting | Default | What it does |
|---|---|---|
| `skip_logos` | true | Skips the warning screen and logos. |
| `skip_network_check` | true | Skips the network check and goes straight to the title menu, offline. |
| `block_network` | true | Blocks all network access, so a modded game can't go online. |
| `quit_item` | true | Gives each character the Easy Go item. |

**For Seamless Co-op,** set `skip_network_check` and `block_network` to `false` and keep the rest. If you run
both Pocket Bonfire and Easy Come, Easy Go, set them in both files: either mod blocking the network blocks it.
Mod Engine blocks it too unless `blockNetworkAccess=0` in `modengine.ini`. With any of these blocking, Steam
closes the game a few seconds after `ds2sc_launcher.exe` starts it. Seamless keeps the game off FromSoftware's
servers itself, but a normal launch through Steam with nothing blocking goes online with mods loaded, so turn
the blocks back on before playing without Seamless.

## Rules and limits

- **There's no confirmation.** Using the item plays a short animation, then saves and quits.
- **It doesn't add a menu entry.** The System menu's rows are fixed parts of its layout, so this is an item
  instead.
- **It uses the game's own Quit Game path.** It saves the way Quit Game does, then closes at the title.
- **The text is in English.** The item's name and description show in English in every language.

## Other mods

The item's data and text are added in memory, so the mod doesn't replace any game files and can't conflict
with mods that change params or text.

**If you also use [Pocket Bonfire](https://github.com/TheMutantFactory/ds2-pocket-bonfire), you need its
version 0.10.1 or later.** Earlier versions can stop one of the two mods working, differently on each
launch.

It has been played alongside Pocket Bonfire, Bonfire Level Up, StatFixer, True Easy Parry, RIP Enemies, No More
Adaptability, Load Loose Param Files, Mod Engine, DS2 Lighting Engine and Texture Renewal Project.

## Uninstall

1. Delete `Game\dllMods\easy_come_easy_go.dll`, `easy_come_easy_go.log` and the `easy_come_easy_go\` folder.
2. If no other mod uses Lazy Loader, delete `Game\dinput8.dll` as well.

Loading a save that still holds the item after removing the mod is untested, so keep your save backup.

## Troubleshooting

`Game\dllMods\easy_come_easy_go.log` records what the mod did. Attach it to bug reports.

- **"signature ... matched 0 times":** the game's code isn't where the mod expects. The line names what it
  found there, and if another mod has hooked that spot, which DLL it belongs to. Please include that line.
- **The item never arrives:** look for "couldn't add the item row" or a disabled line in the log. The quick
  start keeps working either way, since each part starts on its own.

Characters are told apart by name, so two with the same name share a single item. To get a new one, delete
that character's `.item` file from `Game\dllMods\easy_come_easy_go\`.

## Building

You need Rust and the MSVC compiler, plus the Lazy Loader 2.0.0 zip from Nexus:

```powershell
.\scripts\package.ps1 -LazyLoaderZip <path to the Lazy Loader zip>
```

This writes `out\easy-come-easy-go-<version>.zip`.

## Credits

- [DS2S-META](https://github.com/pseudostripy/DS2S-META) by pseudostripy: offsets and signatures.
- [Mod Engine](https://github.com/katalash/ModEngine) by Katalash: the approach to blocking the network.
- [Lazy Loader](https://www.nexusmods.com/darksouls3/mods/677) by Church Guard: loading the mod.
- [MinHook](https://github.com/TsudaKageyu/minhook) by Tsuda Kageyu and the
  [minhook crate](https://github.com/Jakobzs/minhook) by Jakobzs: function hooking.

## License

MIT. See [LICENSE](LICENSE). Release zips include `THIRD-PARTY-NOTICES.txt` for the libraries linked into
the mod, and for Lazy Loader.

---

Don't go hollow, I love you.

https://mutantfactory.net/feedback
