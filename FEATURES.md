# ww_tools features and roadmap

This file is the single source of truth for what the `ww` binary and the `wire_weaver` wheel of this repository
support, what is being worked on and what is planned, for humans and AI agents alike.

## How to use this file

- **Status** of each item:
  - ✅ done
  - 🚧 in progress or partially done (the note says what is missing)
  - 📋 planned
  - 💡 idea, not committed to
  - ⛔ blocked (the note says on what)
  - 🔍 probably done or obsolete, needs a check before closing
  - 🧪 done and tested on the host, the check on hardware is still open (the note says what to check)
  - 🐛 known bug, not yet fixed (the note says how to hit it)
- **Target** is the release an item is planned for (`v0.5` is WireWeaver's next release, which these items were
  planned with). No tag means not scheduled.
- **IDs** (`TOOL-39`) are stable: never renumber or reuse one. New items take the next free number of their area.
  Use the ID in commit messages, CHANGELOG entries and code `TODO`s (`// TODO(TOOL-40): ...`). When talking to
  people, name an ID with a short slug from its title (`TOOL-39 ww-links-own-repos-commands`), never bare.
- The items came from WireWeaver's `FEATURES.md` with their IDs. The areas are WireWeaver's: an item here is one
  whose code lives in this repository. IDs not listed here (`TOOL-36`, `CLT-74`, `EVO-80`, ...) are
  [wire_weaver](https://github.com/vhrdtech/wire_weaver)'s or [ww_stdlib](https://github.com/vhrdtech/ww_stdlib)'s.

## Tools (TOOL)

- ✅ **TOOL-39 `ww` links its own repos' commands** · `v0.5` — the `ww` binary and the `wire_weaver` wheel compile in
  the commands of logring and chunky_mcu_updater (`ww logring`, `ww chunky`, `ww dfu flash`), next to WireWeaver's
  generic commands and the standard library's, so a debug tool works with no plugin; plugins stay for third-party
  tools. A forward (`dfu flash` to `chunky flash`) runs a registered command in process with the device flags typed
  before and after it, and completes with it. This repository is its own workspace above all of them: every crate
  comes by git and `Cargo.lock` holds one copy of each (`ww/tests/it/lock.rs`). Both embed the API snapshots of the
  lock's crates and of every crate of the locked ww_stdlib checkout (`ww_snapshot_set`), so `ww api fetch ww_adc
  0.3.0` answers `embedded in ww` offline. `wire_weaver.logring` in the wheel. Tests: the binary's command list and
  its help, `ww logring` on an image, `ww dfu flash` help and completion with no plugin on PATH, the embedded
  snapshots, pytest with WireWeaver's fixtures from the locked checkout, the release wheel imported from a fresh venv.

- ✅ **TOOL-41 `ww` tracks current wire_weaver** · `v0.5` — `Cargo.lock` takes up wire_weaver (now at `db8f06c`,
  past EVO-82), `ww_stdlib`, `ww_tape` and `shrink_wrap`, and logring's own lock to pick up its matching
  `exact_units` snapshot fix. `ww api save --force` writes the current snapshot model format instead of silently
  downgrading it. `wire_weaver_cli::Program` gained a `hub: ww_hub::HubHost` field for the hub's Captures API; `ww`
  passes `HubHost::default()` (the hub records nothing) until P2401#17nov-2211b sets it to `eventfull_tap`.
