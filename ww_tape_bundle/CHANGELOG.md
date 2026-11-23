# ww_tape_bundle changelog

## Unreleased

### Features

- `stream_layout`, `stream_layout_at`, `StreamLayout`, `StreamKind`, `LayoutError`: a stream's `#[track]` as a ww_tape track kind (API-71); `NamePathError::Empty`.
- `name_path` takes wire_weaver's mount slots (`ApiItemKind::Mount`, SRV-76): a path through a slot ends at its `presence`; `NamePathError::Mount`.
- First version: `name_path`, `key_in_bundle` and `NamePathError`, formerly `ww_tape`'s feature `bundle` (STD-125).
- `key_in_bundle` returns a `ww_tape::TrackKey` (ww_tape READ-5).
- Moved from vhrdtech/wire_weaver to vhrdtech/ww_tools; takes `ww_tape` by git from vhrdtech/ww_tape (STD-125).
