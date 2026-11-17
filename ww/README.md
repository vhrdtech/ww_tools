# ww

The WireWeaver command line tool, as a binary: `wire_weaver_cli`'s generic commands, `ww_std_cli`'s commands of the
standard library APIs, and the commands of the tools built on WireWeaver (`ww logring` from logring, `ww chunky` from
chunky_mcu_updater), linked together and taken by git. It embeds the API snapshots of the whole standard library. The
commands, their flags, the shell completion and the plugins are described in WireWeaver's
[command line reference](https://github.com/vhrdtech/wire_weaver/blob/master/docs/cli_reference.md).

```sh
cargo install --locked --git https://github.com/vhrdtech/ww_tools ww
```

From a checkout, `just install` builds `ww` with the `deploy` profile and installs it; `cargo run -p ww -- <args>` runs
it without installing.
