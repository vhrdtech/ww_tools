# ww_tap_decoder changelog

## Unreleased

### Features

- `define_link_tracks` lays tap tracks out by `LinkTap`'s `#[track]` (`TapLayouts`) and refuses undeclared dimensions; it returns a `Result` (API-71).
- First version: `wire_weaver_client::tap`'s decoder and `ww_tap::tape`, moved from vhrdtech/wire_weaver (TR-14).
