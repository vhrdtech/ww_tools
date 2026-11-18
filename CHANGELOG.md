# ww_tools changelog

The `ww` binary and the `wire_weaver` wheel were developed in the wire_weaver repository (`wire_weaver_cli/`,
`wire_weaver_py/`, then `ww_tools/`) and moved here in November 2026 with their history and feature item. Each crate
keeps its own detailed changelog (`<crate>/CHANGELOG.md`); this file has one summary line per change.

## [Unreleased]

- `ww`: wire_weaver, ww_stdlib, ww_tape and shrink_wrap current; `ww api save` no longer downgrades snapshots (TOOL-41).

## [0.5.0] - 2026-11-17

### Breaking

- Repository: `ww` and the `wire_weaver` wheel live in vhrdtech/ww_tools; install with `cargo install --locked --git https://github.com/vhrdtech/ww_tools ww`.

### Features

- `just install` builds `ww` and the release wheel; `just check` imports the wheel from a fresh venv (TOOL-39).

### Fixes

- `ww` and the wheel embed every standard library API snapshot, `ww_adc` included, not only those of linked crates (TOOL-39).
