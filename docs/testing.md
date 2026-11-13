# Testing

Easy Come, Easy Go is tested together with Pocket Bonfire, since players run them side by side. The runner
and the full matrix live in Pocket Bonfire's repo:
[ds2-pocket-bonfire/docs/testing.md](https://github.com/TheMutantFactory/ds2-pocket-bonfire/blob/develop/docs/testing.md).
Check out both repos next to each other and run, from `ds2-pocket-bonfire`:

```powershell
.\scripts\regression.ps1          # unit tests and signatures for both mods
.\scripts\regression.ps1 -InGame  # plus every in-game scenario, about 25 minutes
```

## No hosted CI

Nothing runs on GitHub: the signature checks need the game's exe and the in-game scenarios need the game,
Steam and a save. The runner is the pipeline, run on the developer's PC before every merge and release.
Releases follow Git Flow (`feature/*` into `develop`, `release/*` or `hotfix/*` into `main`, tagged
`vX.Y.Z`); see [CHANGELOG.md](../CHANGELOG.md).

## What covers this mod

| Check | Where |
|---|---|
| Unit tests: settings files including binary garbage, adding the item's row, the hook lock, a signature matching a hooked function, Seamless launcher detection | `cargo test --workspace`, here or through the runner |
| Every signature found exactly once, also under another mod's hook | Runner, offline |
| Breaking one signature turns off only the feature using it, with an error naming it | Runner, offline |
| Easy Go's name and description show | Every in-game scenario |
| Using Easy Go saves, reaches the title and closes the game | Every in-game scenario |
| Easy Go used mid-roll, with the menu open, right before a quit | `full` |
| Works with another mod's hooks in place first | `hooked` |
| Network check and network left alone with both settings off | `seamless` |
| Seamless Co-op seen with the default settings, network left alone | `coop` |
| No false Seamless detection on a normal launch | Every scenario but `coop` |
| The item given to a character without records | `fresh` |
| Garbage settings fall back to the defaults | `damaged` |

## Last results

2026-11-13: every offline check and every in-game scenario passes, including `coop` under Seamless Co-op
0.0.7 with the default settings (28 of 28).

## Not covered

- A real co-op session with a second player joining.
- Creating a new character; `fresh` simulates one by deleting the records.
- Languages other than English, and any exe other than Steam 1.03.
