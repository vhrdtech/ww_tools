# Changelog

## Unreleased

### Breaking

- Repository: the wheel lives in [vhrdtech/ww_tools](https://github.com/vhrdtech/ww_tools) with its history; `just wheel` there builds it.

### Fixes

- The embedded API snapshots hold every standard library crate, not only those a linked crate depends on (TOOL-39).

### Features

- `wire_weaver.logring`: logring's Python API (`open()`, tracks, `to_pandas()`) in the wheel (TOOL-39).
- New crate (CLT-74): the `wire_weaver` wheel, with its `pyproject.toml`, Python package and tests, moved from `wire_weaver_py`.
