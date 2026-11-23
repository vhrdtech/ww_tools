use super::*;
use crate::test_tape::tap_layouts;
use wire_weaver::shrink_wrap::UNib32;
use ww_tap::{Framer, Medium, Side};

fn session(peer: &str) -> Session<'_> {
    Session {
        side: Side::Host,
        medium: Medium::Usb,
        framer: Framer::None,
        frame_size: UNib32(64),
        peer,
        device_info: RefVec::Slice { slice: &[] },
    }
}

/// Every stream's track is found again from its definition, with the link's identity.
#[test]
fn a_link_s_tracks_are_found_by_their_definitions() {
    let mut defs = Definitions::new();
    let clock = define_clocks(&mut defs, 1_000);
    let identity = LinkIdentity::of(&session("SN-1"), None);
    let tracks = define_link_tracks(&mut defs, clock, &identity, tap_layouts()).unwrap();
    for (stream, track) in tracks.into_iter().enumerate() {
        let (found, s) = link_stream_in(&defs, track).expect("a tap track");
        assert_eq!(found, identity);
        assert_eq!(s as usize, stream);
    }
    assert_eq!(identity.get(dims::PEER), Some("SN-1"));
    assert_eq!(identity.to_string(), "medium=usb peer=SN-1");
}

/// The same device gives the same track ids whatever else a capture holds; another peer gives others.
#[test]
fn a_device_has_the_same_track_ids_in_every_capture() {
    let a = LinkIdentity::of(&session("SN-1"), None);
    let b = LinkIdentity::of(&session("SN-1"), None);
    let other = LinkIdentity::of(&session("SN-2"), None);
    for stream in 0..u32::try_from(STREAMS).unwrap() {
        assert_eq!(a.track_id(stream), b.track_id(stream));
        assert_ne!(a.track_id(stream), other.track_id(stream));
    }
    assert_ne!(
        LinkIdentity::of(&session(""), Some(3)),
        LinkIdentity::numbered(3)
    );
}

/// A records block with a link's Closed event drops that link's tracks from a relay's definitions, not another's.
#[test]
fn a_closed_link_s_tracks_are_dropped() {
    let mut defs = Definitions::new();
    let clock = define_clocks(&mut defs, 0);
    let one =
        define_link_tracks(&mut defs, clock, &LinkIdentity::numbered(0), tap_layouts()).unwrap();
    let two =
        define_link_tracks(&mut defs, clock, &LinkIdentity::numbered(1), tap_layouts()).unwrap();
    let mut closed = [0u8; 8];
    let closed = LinkEvent::Closed.to_ww_bytes(&mut closed).unwrap();
    let mut buf = vec![0; 4096];
    let id = ww_tape::BlockId::new(ww_tape::SessionTag::writable(1), 0);
    let mut w =
        ww_tape::BlockWriter::new(&mut buf, ww_tape::BlockKind::Records, id, &[clock]).unwrap();
    w.push(
        one[ids::EVENTS as usize],
        ww_tape::ClockTime::new(clock, 5),
        &[closed],
    )
    .unwrap();
    let len = w.finish();
    let block = Block::read(&buf[..len]).unwrap();
    assert_eq!(drop_closed_links(&mut defs, &block), 1);
    assert!(link_stream_in(&defs, one[0]).is_none());
    assert!(link_stream_in(&defs, two[0]).is_some());
}

/// Every tap stream is laid out as `LinkTap`'s `#[track]` says (API-71): points, which take the link's dimensions;
/// a dimension the streams don't declare is refused.
#[test]
fn tap_streams_are_points_with_the_link_dims() {
    let layouts = tap_layouts();
    for stream in 0..u32::try_from(STREAMS).unwrap() {
        assert_eq!(layouts.events(stream), ww_tape::EventsDef::INSTANTS);
        assert_eq!(
            layouts.stream(stream).track.dims,
            [dims::LINK, dims::MEDIUM, dims::PEER]
        );
    }
    let mut defs = Definitions::new();
    let clock = define_clocks(&mut defs, 0);
    let stray = LinkIdentity {
        dimensions: vec![("colour".to_owned(), "red".to_owned())],
    };
    let e = define_link_tracks(&mut defs, clock, &stray, layouts)
        .unwrap_err()
        .to_string();
    assert_eq!(
        e,
        "dimension `colour` is not in the `#[track(dims(..))]` of `session`"
    );
    assert_eq!(defs.tracks().count(), 0, "nothing defined");
}

/// A bundle whose `LinkTap` has no `#[track]` (one saved before API-71) gives points without dimensions, so the
/// link's dimensions are refused rather than recorded against an undeclared layout.
#[test]
fn a_tap_bundle_without_tracks_declares_no_dims() {
    let mut bundle = crate::test_resolver()
        .snapshot(&api().make_owned())
        .unwrap()
        .clone();
    for t in &mut bundle.traits {
        if let ApiLevelLocationOwned::InLine { level, .. } = t {
            for item in &mut level.items {
                if let Some(meta) = &mut item.meta {
                    meta.track = None;
                }
            }
        }
    }
    let layouts = TapLayouts::from_bundle(&bundle).unwrap();
    assert_eq!(layouts.events(ids::EVENTS), ww_tape::EventsDef::INSTANTS);
    let mut defs = Definitions::new();
    let clock = define_clocks(&mut defs, 0);
    assert!(define_link_tracks(&mut defs, clock, &LinkIdentity::numbered(0), &layouts).is_err());
}
