# ww_tap_decoder changelog

## Unreleased

### Features

- `define_link_tracks` lays tap tracks out by `LinkTap`'s `#[track]` (`TapLayouts`) and refuses undeclared dimensions; it returns a `Result` (API-71).
- Tests take `ww_quantity`, where wire_weaver moved the dynamic quantities.
- First version: `wire_weaver_client::tap`'s decoder and `ww_tap::tape`, moved from vhrdtech/wire_weaver (TR-14).
