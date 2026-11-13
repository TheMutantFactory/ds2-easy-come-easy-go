# Building on this

For anyone forking Easy Come, Easy Go, reimplementing it, or writing their own DS2 DLL mod. The code is
MIT, so take what helps.

Most of what we learned applies to both of our mods and is written up once, in Pocket Bonfire's
[docs/building-on-this.md](https://github.com/TheMutantFactory/ds2-pocket-bonfire/blob/develop/docs/building-on-this.md):
the game's quirks, adding items without changing game files, living with other mods' hooks, the network
and Seamless Co-op, and testing in the real game. This repo carries a copy of the same `crates/ds2`.

If you build something on it, we'd love to hear about it: the Nexus Mods thread or
https://mutantfactory.net/feedback.

## Specific to this mod

- **There's no System menu entry on purpose.** The System menu's rows are fixed parts of its layout, and
  an item needs no game files at all.
- **Quitting reuses the game's own Quit Game path,** which saves the way the menu does. The mod then waits
  until the game has sat at the title for a couple of seconds, so the world has finished unloading, before
  closing it.
- **Signatures must match exactly once.** The first closing signature matched two functions, which would
  have switched the mod off. `scan_exe` catches this.
- **The quick start and the item start separately.** If the item's signatures break, the quick start
  still works, and the other way round. `quit_item = false` leaves only the quick start.
- **The item's row id is `60595010`.** Pick a different one for your own items.
- **It shares Pocket Bonfire's hook lock** (`Local\DS2-TheMutantFactory-Hooks`), since both mods hook
  `ApplySpEffect` and `FmgLookup`. Without it, either could lose its hooks depending on timing.

## Caveats

- Only Steam SotFS 1.03 is targeted; on any other exe the mod turns off everything except the network
  block.
- Offsets and signatures come from [DS2S-META](https://github.com/pseudostripy/DS2S-META) (MIT).
- Not tested: a real co-op session with a second player, a freshly created character (only simulated), and
  languages other than English.

---

Don't go hollow, I love you.

https://mutantfactory.net/feedback
