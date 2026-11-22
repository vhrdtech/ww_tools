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

- 🚧 **TOOL-42 `ww` tracks current wire_weaver past SRV-64** · `v0.5` — `Cargo.lock` takes up wire_weaver at
  `383e4bf0` (hub connection counting at accept, poison-safe capture guards, snapshot-dependency checks past SRV-64);
  `ww_stdlib`, logring and chunky_mcu_updater were already current. `wire_weaver_cli::Program::hub` and
  `wire_weaver_py::set_hub_host` (SRV-64's `CaptureHost`) are unchanged from TOOL-41: no `CaptureHost` exists yet in
  wire_weaver, `ww_stdlib` or logring (wire_weaver's own `FEATURES.md` SRV-64 entry: "Missing: eventfull's capture
  host (P2555 piece 8)"), so `ww` and the wheel still give their hub no capture host and it records nothing. Missing:
  wiring eventfull's capture host once it exists (P2401#17nov-2211b, P2555 piece 8).

## Transport (TR)

- ✅ **TR-14 link tap capture decoder** · `v0.5` — the reading side of wire_weaver's link tap (TR-14 there: the `Tap`
  hook, `TapSource`, `ww_tap`'s records) moved here with the item's id, so wire_weaver has no `ww_tape` dependency.
  `ww_tap_decoder`: frame ↔ message linkage, cross-check against the message records, framer and link statistics
  (`Stats`), request/reply pairing by seq, calls and replies named and explained from the API bundle a capture
  carries or from snapshots (global traits by `ww_global` id on a traits-only connection, API-20), a session's wire
  version (`session_wire_version`). Its `tape` module names a link's `ww_tape` tracks and the tap clock
  (`define_link_tracks`, `define_clocks`, `track_key`) and finds them again (`link_stream_of`). Input: records
  (`Decoder::record`, `session`, `gap`) or blocks (`feed`). Tests: a dynamic client against `ww_device::Server` over
  simulated USB with a capture writer of the tests, shared packets, split replies, a lost packet (with and without
  the frame counter), captures replayed identically, a global trait named on full and traits-only connections, a
  quantity whose unit is sent by reference decoded from the sent bundle alone.

## Standard library (STD)

- ✅ **STD-125 track identity by name path, the API side** · `v0.5` — `ww_tape_bundle` (`name_path`,
  `key_in_bundle`) and its tests (`tests/tape_identity`, over wire_weaver's fixtures from the checkout `Cargo.lock`
  places) moved here from wire_weaver; the tape format's side is in vhrdtech/ww_tape under the same id.
