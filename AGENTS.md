# AGENTS.md

Guidance for AI code assistants working in this repository.

## What this is

ww_tools holds the shipped tools of [WireWeaver](https://github.com/vhrdtech/wire_weaver) (layer 7 of its
architecture): the one `ww` binary and the one `wire_weaver` Python wheel. Both compile in every command and module
of WireWeaver (`wire_weaver_cli`, `wire_weaver_py`), of its standard library (`ww_std_cli` from ww_stdlib), of logring
(`logring_ww_cli`, `logring_py`) and of chunky_mcu_updater (`chunky_ww_cli`). Nothing in this repository defines a
command: commands register themselves in their own crates (inventory, TOOL-36), and linking the crate adds them.
A debug tool must work without being debugged itself: `ww` needs no plugin, the wheel nothing else installed.

Nothing depends on this repository. A change in the crates it links lands in their repository first, then comes
here with `cargo update -p NAME`.

## Layout

| Path | What |
| --- | --- |
| `ww/` | the binary: `src/main.rs` links the command crates and embeds the snapshot set; `build.rs` makes `--version` |
| `ww/tests/it/` | the integration tests, one binary: command list, help, logring and chunky with no plugin, hub, RTT, plugin probe, embedded snapshots, lock |
| `wire_weaver_wheel/` | the wheel: `src/lib.rs` (the `_wire_weaver` module), `python/wire_weaver/` (the package), `tests/` (pytest), `pyproject.toml` |
| `wire_weaver_wheel/tests/wire_weaver_checkout.py` | finds the locked wire_weaver checkout: the Python tests read its fixtures and build its simulated devices |
| `ww_snapshot_set/` | build dependency: the API snapshots of the lock's crates and of every crate in the locked ww_stdlib checkout |

## FEATURES.md

`FEATURES.md` is the feature tracker, the only place status is kept. Check it before starting a feature and update
the matching item in the same change (conventions at its top).

- IDs are stable per area (`TOOL-39`): never renumber or reuse one; new items take the next free number of their
  area. Use them in commits, CHANGELOG entries and `// TODO(TOOL-40): ...`.
- When talking to the user, name an ID with a 2-4 word kebab-case slug: `TOOL-39 ww-links-own-repos-commands`.
- Items came from WireWeaver's tracker with their IDs; an ID not listed here is WireWeaver's or its standard library's.

## Commands

```sh
just test --changed   # Rust tests of changed crates and their dependents, then the Python tests
just test             # everything: nextest, doctests, pytest, just check
just lint             # fmt, cargo sort -w -g, taplo, typos, clippy and rustdoc -D warnings, cargo machete
just check            # the release wheel, imported from a fresh venv
just test-py          # the Python tests (uv builds the module into wire_weaver_wheel/.venv)
just wheel            # the release wheel into target/wheels
just install          # ww on PATH (through tpm when installed, else cargo install) and the release wheel
just py               # Python console with the device connected as `dev`
```

Single test: `cargo nextest run -p ww <test>` (nextest is required); `cd wire_weaver_wheel && uv run --group dev pytest
-q tests/test_hub.py`. Every Python step goes through uv. Plain `cargo sort` flattens grouped tables: always pass
`-w -g`. Project words go into `typos.toml`. CI fails on any warning.

## Tests

- `help_lists_every_command_in_order` (`ww/tests/it/commands.rs`) holds the expected command list. A command added in
  a linked crate shows up here after `cargo update -p NAME`: add it to the list in the same commit.
- `stdlib_snapshots_are_embedded` checks that `ww api fetch ww_adc 0.3.0` answers `embedded in ww` with an empty Cargo
  home and `WW_OFFLINE=1`.
- The Python tests use WireWeaver's fixtures (`tests/conformance`, `tests/*_api`, `hub_pytest_device`,
  `thermo_devices`) from Cargo's checkout of the locked wire_weaver commit; the devices are built from it with its own
  lock into `target/wire_weaver_fixtures`. A fixture changes in wire_weaver, never here.
- Tests run on a scratch WireWeaver home with no fetch (`.config/test-home.sh`); none reads `~/.wire_weaver`.

## Dependencies

- Every crate outside this repository comes by git, no `rev` or `tag`: wire_weaver crates from vhrdtech/wire_weaver,
  `ww_std_cli` from vhrdtech/ww_stdlib, `logring_*` from vhrdtech/logring, `chunky_ww_cli` from
  romixlab/chunky_mcu_updater. Their own dependencies (shrink_wrap, exact_units, ww_base, ww_tape) come with them.
- `Cargo.lock` holds one copy of each of these crates (`ww/tests/it/lock.rs` checks it). Two copies mean two
  registries of commands or two sets of types: update the repository that lags first.
- No `[patch]` is committed. To develop against local checkouts, put a `[patch."https://github.com/..."]` in a
  `.cargo/config.toml` above this checkout and commit no `Cargo.lock` that names a patched path.
- A dependency only tests use is a dev-dependency; `cargo machete` runs in `just lint`.

## Proper fixes, never a lost capability

- A bug or limit found on the way is fixed where it lives, as its own commit, with a test that fails without it. A
  bug in a linked crate is fixed in its repository, then taken here with `cargo update -p NAME`.
- A removal, replacement or refactor keeps everything that worked before. Never narrow tests, docs or
  `FEATURES.md` to fit a loss.
- Flaky and slow tests: find the cause, never raise a limit (`.config/nextest.toml` holds hang detectors only).

## Changelogs, commits and versions

- Every user-visible change gets an entry under `## Unreleased` in the `CHANGELOG.md` of each affected crate, in the
  same change (`### Breaking`, `### Features`, `### Fixes`; for breaking changes say what users must change). The root
  `CHANGELOG.md` gets one summary line per change.
- Commits use Conventional Commits with a scope (`fix(ww): ...`): a short imperative subject, a blank line, a body of
  at most 5 lines saying what and why, and a `Tested:` line. A move is never edited in the same commit.
- Work commits never bump versions; the release commit does. The version follows WireWeaver's (`0.5.0`).
- Dependencies live in the root `[workspace.dependencies]`; crates inherit `authors`, `edition`, `rust-version`,
  `license` and `repository` from `[workspace.package]` and take `[lints] workspace = true`.
- Public text (README, docs, CHANGELOG, commits, comments) is neutral: no private paths or internal tracker names.
