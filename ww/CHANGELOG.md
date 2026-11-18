# Changelog

## Unreleased

### Breaking

- Repository: `ww` lives in [vhrdtech/ww_tools](https://github.com/vhrdtech/ww_tools) with its history; install with `cargo install --locked --git https://github.com/vhrdtech/ww_tools ww`.
- Every crate comes by git with no `[patch]`: wire_weaver's from vhrdtech/wire_weaver, `ww_std_cli` from vhrdtech/ww_stdlib, logring's and chunky_mcu_updater's from their repositories; `Cargo.lock` holds one copy of each.

### Features

- `ww logring` and `ww chunky` are built in, and `ww dfu flash` runs chunky's flash in process: no plugin needed (TOOL-39).
- New crate (TOOL-36): the `ww` binary, `wire_weaver_cli` and `ww_std_cli` linked; `cargo install --git .. ww`.
- `wire_weaver_cli::Program` takes a `hub: ww_hub::HubHost`; `ww` passes `HubHost::default()` (the hub records nothing) until P2401#17nov-2211b sets it to `eventfull_tap` (TOOL-41).

### Fixes

- `Cargo.lock` takes up wire_weaver (past EVO-82), `ww_stdlib`, `ww_tape` and `shrink_wrap`, and logring's own lock to pick up its matching `exact_units` fix: `ww api save --force` writes the current snapshot model format instead of silently downgrading it (TOOL-41).
