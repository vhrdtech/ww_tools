# ww_tools

[![CI](https://github.com/vhrdtech/ww_tools/actions/workflows/ci.yml/badge.svg)](https://github.com/vhrdtech/ww_tools/actions/workflows/ci.yml)

<p align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/vhrdtech/wire_weaver/master/docs/assets/wire_weaver_logo-dark.png">
  <img src="https://raw.githubusercontent.com/vhrdtech/wire_weaver/master/docs/assets/wire_weaver_logo.png" alt="WireWeaver logo" width="480"/>
</picture>
</p>

> The shipped tools of [WireWeaver](https://github.com/vhrdtech/wire_weaver): the `ww` command line tool and the
> `wire_weaver` Python wheel, with every command built in.

`ww` and the wheel link the generic commands and the Python module of WireWeaver, the device commands of its standard
library ([ww_stdlib](https://github.com/vhrdtech/ww_stdlib)), and the commands of the tools built on it:
`ww logring` and `wire_weaver.logring` from [logring](https://github.com/vhrdtech/logring), `ww chunky` from
[chunky_mcu_updater](https://github.com/romixlab/chunky_mcu_updater). Neither needs a plugin or anything else
installed. Both embed the API snapshots of the whole standard library, so a device built with any standard API
resolves offline.

## Install

```sh
cargo install --locked --git https://github.com/vhrdtech/ww_tools ww
```

From a checkout, `just install` builds `ww` with the `deploy` profile and puts it on `PATH`, and builds the release
wheel into `target/wheels/` (`uv pip install target/wheels/wire_weaver-*.whl` or `uv add` it in a project).

## Crates

| Crate | What |
| --- | --- |
| `ww` | the `ww` binary: `wire_weaver_cli`, `ww_std_cli`, `logring_ww_cli` and `chunky_ww_cli` linked together |
| `wire_weaver_wheel` | the `wire_weaver` Python wheel: `wire_weaver_py` and `logring_py` as one extension module, built by maturin |
| `ww_snapshot_set` | the API snapshot set both embed: the lock's crates and every crate of the locked ww_stdlib |
| `ww_tap_decoder` | decoder of link tap captures: frames, messages and API calls from `ww_tape` blocks, with link stats |
| `ww_tape_bundle` | name paths and track keys of `ww_tape` tracks from an API bundle |

## Documentation

The [WireWeaver book](https://ww.vhrd.tech/) describes the commands, their flags, completion and plugins in the
[command line reference](https://ww.vhrd.tech/cli_reference/), and the Python module in the
[Python page](https://ww.vhrd.tech/python/). [`FEATURES.md`](FEATURES.md) tracks what is done and planned.

## Development

`just --list` shows the recipes: `just test`, `just lint`, `just check` (the release wheel, imported from a fresh
venv), `just install`. The other repositories come by git; `Cargo.lock` pins each commit and
`cargo update -p NAME` moves it.

## License

MIT, see [LICENSE](LICENSE).
