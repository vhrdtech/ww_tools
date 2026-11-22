# Changelog

## Unreleased

### Breaking

- `ww monitor`, `ww stats`, `ww capture` and `ww explain FILE` are gone with wire_weaver's tap recorder (TOOL-14).

- Repository: `ww` lives in [vhrdtech/ww_tools](https://github.com/vhrdtech/ww_tools) with its history; install with `cargo install --locked --git https://github.com/vhrdtech/ww_tools ww`.
- Every crate comes by git with no `[patch]`: wire_weaver's from vhrdtech/wire_weaver, `ww_std_cli` from vhrdtech/ww_stdlib, logring's and chunky_mcu_updater's from their repositories; `Cargo.lock` holds one copy of each.

### Features

- `ww logring` and `ww chunky` are built in, and `ww dfu flash` runs chunky's flash in process: no plugin needed (TOOL-39).
- New crate (TOOL-36): the `ww` binary, `wire_weaver_cli` and `ww_std_cli` linked; `cargo install --git .. ww`.
- `wire_weaver_cli::Program` takes a `hub: ww_hub::HubHost`; `ww` passes `HubHost::default()` (records nothing) (TOOL-41).

### Fixes

- The embedded API snapshots hold every standard library crate (`ww_adc`, `ww_spi`, ...), not only those a command links: `ww api fetch ww_adc 0.3.0` answers `embedded in ww` offline (TOOL-39).
- `ww api save --force` no longer downgrades snapshots to an older model format (TOOL-41).
- `Cargo.lock` takes up wire_weaver at `383e4bf0` (past SRV-64's follow-up fixes); still no `CaptureHost` to give the hub, so `ww` keeps passing `HubHost::default()` (TOOL-42).
