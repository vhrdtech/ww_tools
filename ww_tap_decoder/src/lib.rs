//! Decoder of WireWeaver link tap captures (TR-14): what the [Tap](ww_tap::Tap) of a client's link reported, written
//! as `ww_tape` blocks on the tracks of [tape], read back as frames, messages and API calls ([Decoder]) with link
//! statistics ([Stats]). The tap hook itself ([TapSource](wire_weaver_client::tap::TapSource)) is in
//! `wire_weaver_client` and needs no `ww_tape`.

pub mod decode;
mod reassembly;
pub mod stats;
pub mod tape;

pub use decode::{
    ApiInfo, Check, Decoded, Decoder, DecoderConfig, FrameInfo, MessageInfo, What,
    session_wire_version,
};
pub use reassembly::{FrameOutcome, FramePart, PieceInfo, PieceOutcome, Reassembled};
pub use stats::{DirStats, LinkStats, PathStats, PrefixCounts, Stats};
pub use tape::*;
pub use ww_framer::prefix::{FrameCtx, FrameReport};

#[cfg(test)]
mod e2e_tests;
#[cfg(test)]
mod test_tape;

/// The resolver of the tests: snapshots from Cargo's sources of this workspace only, never fetched.
#[cfg(test)]
fn test_resolver() -> &'static ww_snapshots::Resolver {
    static RESOLVER: std::sync::LazyLock<ww_snapshots::Resolver> = std::sync::LazyLock::new(|| {
        ww_snapshots::Resolver::offline(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
    });
    &RESOLVER
}
