# ww_snapshot_set

The API snapshot set that `ww` and the `wire_weaver` wheel embed, written by their build scripts.

It holds the snapshots of every crate in this workspace's `Cargo.lock` and of every standard library API crate in the `ww_stdlib` checkout the lock pins, whether `ww` depends on that crate or not. A device built with any standard library API then resolves offline: `WW_OFFLINE=1 ww api fetch ww_adc 0.3.0` reports `embedded in ww`.

```rust,ignore
// build.rs
ww_snapshot_set::write("stdlib_set.rs").unwrap_or_else(|e| panic!("embedding API snapshots: {e:#}"));
```
