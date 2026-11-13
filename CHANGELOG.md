# Changelog

Versions follow the release tags. Unreleased work is on `develop` or a `feature/*` branch until it's
merged and tagged.

## 0.3.0 (unreleased)

Holding up better when other mods change the game's code. The same changes as Pocket Bonfire 0.12.0,
in the shared code.

### Added

- **Functions are found by their known address** when another mod changed them so much that no
  signature matches. Only on the exact 1.03 exe, and only if the code there still looks like the
  function. The log says when this happens and which DLL changed it.
- **The log lists the other mods,** at startup and half a minute later, with versions and folders.
- **A watch on our hooks:** if a mod that loads later writes over one, the log names the DLL it now leads
  into.

### Fixed

- The logo skip's flag address could have been read wrong behind a long hook jump. It's now checked, and
  only the logo skip switches off if it can't be found.

### Testing

- Both mods put their items back when another mod swaps the item table mid-game (Pocket Bonfire's
  `hooked` scenario).

## 0.2.1 (unreleased)

### Fixed

- **No Easy Go item with DS2 Inventory Overhaul installed** (Inventory Suite, Nexus 1811). The log said
  `signature ItemGive matched 0 times`. That mod hooks the game's item-giving function with a 14-byte
  jump padded to 16 bytes, and the mod only looked past the first 14. It now recognises the hook jump
  and its padding, whatever the length, and finds the function behind it. Thanks to ElPachu24 for the
  report and the log.

### Testing

- Played with Inventory Suite 1.0.1 installed, in Pocket Bonfire's new `inventory` scenario. The offline
  check now hooks every function with 5-, 12- and 14-byte jumps, with and without padding.

## 0.2.0 (2026-11-13)

Works with Pocket Bonfire 0.10.1 or later. If you also play Seamless Co-op, update Pocket Bonfire to
0.11.0 too: either mod blocking the network is enough to break Seamless.

### Added

- `easy_come_easy_go.ini`, written on first launch: turns the logo skip, the network check skip, the
  network block and the Easy Go item on or off. A missing, unreadable or garbled file falls back to the
  defaults, and the log says which lines it ignored.
- Seamless Co-op support: when `ds2sc_launcher.exe` starts the game, or Seamless's `ds2sc.dll` is loaded,
  the mod leaves the network check and the network alone, whatever the settings say. Seamless needs both,
  and keeps the game off FromSoftware's servers itself. A normal Steam launch stays offline.
- Log lines that say which settings were used, whether Seamless was seen, and network starts refused or
  let through.

### Fixed

- **0.1.0 crashes under Seamless Co-op.** It always refused to start Windows networking, and Steam closes
  the game a few seconds after the Seamless launcher starts it. Fixed by the Seamless support above.
- Works when another mod has hooked the same functions first: each function is still found, and when a
  signature fails, the log says what's there and which DLL a hook leads to.
- The quick start and the item start separately, so one failing leaves the other working.
- Saving and quitting, and giving the item, run on the game's main thread instead of the mod's own.
- Item tables that carry row names are accepted.

### Testing

- Tested with Pocket Bonfire's runner, which covers both mods. See [docs/testing.md](docs/testing.md).

## 0.1.0 (2026-11-05)

First public release.

- Easy come: skips the logos and the network check, and goes straight to the title menu, offline. The
  network is blocked so a modded game can't go online.
- Easy go: a reusable item each character gets once. Using it saves and quits to the desktop.
- The item and its text are added in memory, so no game files change.
- Takes turns with Pocket Bonfire when installing hooks, so neither knocks the other's out.
- Ships as one zip with Lazy Loader.

---

Don't go hollow, I love you.

https://mutantfactory.net/feedback
