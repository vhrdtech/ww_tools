//! Embeds the API snapshots of every crate in the workspace's lock and of the whole standard library (ww_stdlib), so
//! the wheel understands a device built with them without their sources at hand (EVO-80).

fn main() {
    println!("cargo:rerun-if-changed=../Cargo.lock");
    ww_snapshot_set::write("stdlib_set.rs")
        .unwrap_or_else(|e| panic!("embedding API snapshots: {e:#}"));
}
