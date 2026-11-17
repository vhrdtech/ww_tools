//! The lock links one copy of every crate taken by git: two copies of a WireWeaver crate would make two command
//! registries or two sets of the same types.

use std::collections::BTreeMap;

#[test]
fn one_copy_of_each_git_crate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.lock");
    let text = std::fs::read_to_string(path).expect("Cargo.lock is readable");
    let lock: toml::Table = text.parse().expect("Cargo.lock is TOML");
    let packages = lock["package"].as_array().expect("[[package]] entries");
    let mut sources: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for package in packages {
        let source = package.get("source").and_then(|s| s.as_str());
        if let Some(source) = source.filter(|s| s.starts_with("git+")) {
            let name = package["name"].as_str().expect("a package name");
            sources.entry(name).or_default().push(source);
        }
    }
    let twice: Vec<_> = sources.iter().filter(|(_, s)| s.len() > 1).collect();
    assert!(
        twice.is_empty(),
        "more than one copy: {twice:#?}; update the lagging repository"
    );
    assert!(sources.contains_key("wire_weaver"), "{sources:#?}");
}
