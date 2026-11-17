//! The API snapshot set that `ww` and the `wire_weaver` wheel embed, written from their build scripts.
//!
//! The set holds the snapshots of every crate in the workspace's `Cargo.lock`, as
//! [`ww_snapshots::build::workspace_set`] writes them, and of every crate at the top of the `ww_stdlib` checkout the
//! lock pins. The standard library API crates that no command depends on (`ww_adc`, `ww_spi`, ...) are then embedded
//! too, so a device built with any of them resolves offline.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use ww_snapshots::build::Written;
use ww_snapshots::cargo::CargoSources;
use ww_snapshots::{Dir, SNAPSHOTS_DIR};

/// A crate of `ww_stdlib` the lock pins: its checkout's top directory holds every standard library API crate.
const STDLIB_ANCHOR: &str = "ww_std_cli";

/// Upper bound on the entries read from the top of the `ww_stdlib` checkout.
const STDLIB_ENTRIES_MAX: usize = 1_000;

/// Writes the set to `OUT_DIR/<file_name>`; the binary includes it with
/// `include!(concat!(env!("OUT_DIR"), "/<file_name>"))`. Called from a build script.
///
/// # Errors
///
/// Fails when the environment of a build script is missing, the lock does not pin exactly one `ww_std_cli` or a
/// snapshot fails its check.
pub fn write(file_name: &str) -> Result<Written> {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR is not set")?,
    );
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").context("OUT_DIR is not set")?);
    let mut sources = CargoSources::default();
    sources.entry_dir = Some(manifest_dir);
    let mut packages = sources.locked_package_dirs()?;
    packages.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let mut crate_dirs: Vec<PathBuf> = packages
        .into_iter()
        .map(|(_, _, dir)| dir)
        .filter(|dir| Dir::of_crate(dir).exists())
        .collect();
    let anchor = sources.locked_package_dir(STDLIB_ANCHOR)?;
    let stdlib_root = anchor
        .parent()
        .with_context(|| format!("{}: no parent directory", anchor.display()))?;
    crate_dirs.extend(crates_with_snapshots(stdlib_root)?);
    crate_dirs.dedup();
    ww_snapshots::build::write_set(&out_dir, file_name, &crate_dirs)
}

/// The directories right below `root` that have an `api_snapshots/`, sorted by name.
fn crates_with_snapshots(root: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = vec![];
    let entries = fs::read_dir(root).with_context(|| root.display().to_string())?;
    for entry in entries.take(STDLIB_ENTRIES_MAX) {
        let path = entry.with_context(|| root.display().to_string())?.path();
        if path.join(SNAPSHOTS_DIR).is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crates_with_snapshots_are_found_sorted() {
        let root = std::env::temp_dir().join(format!("ww_snapshot_set_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for dir in ["b_api/api_snapshots", "a_api/api_snapshots", "no_api/src"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        let found = crates_with_snapshots(&root).unwrap();
        assert_eq!(found, vec![root.join("a_api"), root.join("b_api")]);
        fs::remove_dir_all(&root).unwrap();
    }
}
