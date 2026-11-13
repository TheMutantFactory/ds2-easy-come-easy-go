# Nexus page draft

Draft for the Dark Souls II page. Publishing is manual: paste these fields, upload the zip from
`scripts/package.ps1`, and add screenshots.

| Field | Value |
|---|---|
| Name | Easy Come, Easy Go |
| Author | mutantfactory |
| Version | 0.3.0 |
| Category | Gameplay |
| Requirements | None, Lazy Loader is included |
| Online | Offline: the mod blocks network access, except under Seamless Co-op, which it detects |
| Source | https://github.com/TheMutantFactory/ds2-easy-come-easy-go |

## Summary

Skip the logos and the network check on the way in. Use one foul item to save and quit to the desktop on the way out.

## Description (BBCode)

```bbcode
[size=5][b]Easy Come, Easy Go[/b][/size]

Gets you in and out of Dark Souls II quickly.

[list]
[*][b]Easy come:[/b] the game skips the logos and the network check, and goes straight to the title menu.
[*][b]Easy go:[/b] a new reusable item with a familiar, unpleasant icon. Use it to save your game and quit to the desktop, without going through the System menu and the title screen.
[/list]
Every character gets one Easy Go the first time you load it after installing. Put it in an item slot to use it.

[b]This mod keeps the game offline.[/b] While it's installed, the game can't connect to its servers, so there's no multiplayer, messages or summoning. Started through Seamless Co-op's launcher, it leaves the network alone by itself. If you use Mod Engine, set blockNetworkAccess=0 in modengine.ini for Seamless, and back to 1 for normal play.

[b]Help us find what breaks.[/b] Hit a bug, or found a mod that doesn't get along with this one? Tell us in the Posts tab here or at [url=https://mutantfactory.net/feedback]mutantfactory.net/feedback[/url]. Which other mods you run, and Game\dllMods\easy_come_easy_go.log from a launch where it went wrong, help most. We can't beat the second law of thermodynamics, but we can fix what breaks if we can.

[b]Install[/b]
You need the Steam version of Scholar of the First Sin, 1.03. On any other exe, the mod logs that and turns off everything except the network block.
[list=1]
[*]Back up your saves, from %APPDATA%\DarkSoulsII.
[*]If Game\dinput8.dll already exists and came from a mod other than Lazy Loader, rename it to something like othermod.dll and move it to Game\dllMods\, creating the folder if needed.
[*]Extract the zip into the game's Steam folder, the one that contains Game\. Vortex's DS2 support deploys to this folder too.
[/list]
That's all. The zip includes [url=https://www.nexusmods.com/darksouls3/mods/677]Lazy Loader[/url], which loads the mod. If you already use it, your lazyLoad.ini isn't touched.

[b]Rules and limits[/b]
[list]
[*]There's no confirmation. Using the item plays a short animation, then saves and quits.
[*]It doesn't add a menu entry. The System menu's rows are fixed parts of its layout, so this is an item instead.
[*]It uses the game's own Quit Game path. It saves the way Quit Game does, then closes at the title.
[*]The item's name and description show in English in every language.
[/list]

[b]Other mods[/b]
The item's data and text are added in memory, so the mod doesn't replace any game files and can't conflict with mods that change params or text.

[b]If you also use Pocket Bonfire, you need its version 0.10.1 or later.[/b] Earlier versions can stop one of the two mods working, differently on each launch.

Played alongside Pocket Bonfire, Bonfire Level Up, StatFixer, True Easy Parry, RIP Enemies, No More Adaptability, Load Loose Param Files, Mod Engine, DS2 Lighting Engine and Texture Renewal Project.

[b]Uninstall and bug reports[/b]
Uninstall steps are in the included README. Loading a save that still holds the item after removing the mod is untested, so keep your save backup.

For bug reports, attach Game\dllMods\easy_come_easy_go.log.

[b]Credits[/b]
[list]
[*][url=https://github.com/pseudostripy/DS2S-META]DS2S-META[/url] by pseudostripy: offsets and signatures
[*][url=https://github.com/katalash/ModEngine]Mod Engine[/url] by Katalash: the approach to blocking the network
[*][url=https://www.nexusmods.com/darksouls3/mods/677]Lazy Loader[/url] by Church Guard: loading the mod
[*][url=https://github.com/TsudaKageyu/minhook]MinHook[/url] by Tsuda Kageyu and the minhook crate by Jakobzs: function hooking
[/list]

Source and license (MIT): [url=https://github.com/TheMutantFactory/ds2-easy-come-easy-go]GitHub[/url]

Don't go hollow, I love you.
[url=https://mutantfactory.net/feedback]https://mutantfactory.net/feedback[/url]
```
