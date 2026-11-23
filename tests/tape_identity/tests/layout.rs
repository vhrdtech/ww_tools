//! Stream layouts on a tape from `#[track(..)]` (API-71) over a real bundle: wire_weaver's `tests/track_api`.

use ww_self::{ApiBundleOwned, ApiLevelOwned};
use ww_tape::{Ends, EventsDef};
use ww_tape_bundle::{LayoutError, NamePathError, StreamKind, stream_layout};

/// The API `root` of the crate at `dir` in the wire_weaver repository: the checkout this workspace's `Cargo.lock`
/// places (Cargo's git checkout, or a local `[patch]`).
fn bundle(dir: &str, root: &str) -> ApiBundleOwned {
    let mut sources = ww_snapshots::cargo::CargoSources::default();
    sources.entry_dir = Some(env!("CARGO_MANIFEST_DIR").into());
    let wire_weaver = sources.locate_crate("wire_weaver").unwrap();
    let repo = wire_weaver[0].parent().unwrap();
    wire_weaver_core::load(&repo.join(dir), Some(root.into()), false).unwrap()
}

fn id(level: &ApiLevelOwned, ident: &str) -> u32 {
    level.items.iter().find(|i| i.ident == ident).unwrap().id.0
}

fn kind(b: &ApiBundleOwned, path: &[u32]) -> StreamKind {
    stream_layout(b, path).unwrap().kind
}

/// Each layout maps to its ww_tape track kind: points and spans to events, a series to its channels.
#[test]
fn layouts_map_to_track_kinds() {
    let b = bundle("tests/track_api", "Track");
    let r = &b.root;
    assert_eq!(
        kind(&b, &[id(r, "events")]),
        StreamKind::Events(EventsDef::INSTANTS)
    );
    assert_eq!(
        kind(&b, &[id(r, "watch")]),
        StreamKind::Events(EventsDef::INSTANTS)
    );
    assert_eq!(
        kind(&b, &[id(r, "detections")]),
        StreamKind::Events(EventsDef {
            ends: Ends::Spans,
            flags: true
        })
    );
    assert_eq!(
        kind(&b, &[id(r, "levels")]),
        StreamKind::Series { channels: 2 }
    );
    assert_eq!(
        kind(&b, &[id(r, "probes")]),
        StreamKind::Series { channels: 4 }
    );
    let detector = r.items.iter().find(|i| i.ident == "detector").unwrap();
    let hits = id(detector.get_as_level(&b).unwrap(), "hits");
    assert_eq!(
        kind(&b, &[id(r, "detector"), hits]),
        StreamKind::Events(EventsDef {
            ends: Ends::Spans,
            flags: false
        })
    );
}

/// A writer gives only the dimensions a stream declares.
#[test]
fn only_declared_dimensions() {
    let b = bundle("tests/track_api", "Track");
    let events = stream_layout(&b, &[id(&b.root, "events")]).unwrap();
    events.check_dimensions(["producer", "input"]).unwrap();
    events.check_dimensions([]).unwrap();
    let e = events
        .check_dimensions(["input", "platform"])
        .unwrap_err()
        .to_string();
    assert_eq!(
        e,
        "dimension `platform` is not in the `#[track(dims(..))]` of `events`"
    );
    let probes = stream_layout(&b, &[id(&b.root, "probes")]).unwrap();
    assert!(
        probes.check_dimensions(["input"]).is_err(),
        "no dims declared"
    );
}

#[test]
fn no_stream_no_layout() {
    let b = bundle("tests/track_api", "Track");
    assert!(matches!(
        stream_layout(&b, &[id(&b.root, "detector")]),
        Err(LayoutError::NotAStream { name }) if name == "detector"
    ));
    assert!(matches!(
        stream_layout(&b, &[99]),
        Err(LayoutError::Path(NamePathError::NoItem { at: 0, id: 99 }))
    ));
    assert!(matches!(
        stream_layout(&b, &[]),
        Err(LayoutError::Path(NamePathError::Empty))
    ));
    assert!(matches!(
        stream_layout(&b, &[id(&b.root, "events"), 0]),
        Err(LayoutError::Path(NamePathError::TooLong { at: 1 }))
    ));
}
