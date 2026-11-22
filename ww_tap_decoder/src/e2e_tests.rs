//! The tap end to end (TR-14): a dynamic client against `ww_device::Server` over simulated USB (64 B packets, real
//! framers on both ends), recorded into a capture's blocks, decoded from them and again from the capture file.

use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};

use tokio::sync::{Notify, oneshot};
use wire_weaver::shrink_wrap::{DeserializeShrinkWrap, DeserializeShrinkWrapOwned};
use wire_weaver_client::in_process::{Disconnected, TokioClock, UsbSink, usb_device_framed};
use wire_weaver_client::ww_numeric::NumericValue;
use wire_weaver_client::ww_self::ValueOwned;
use wire_weaver_client::{ClientConfig, DynClient, DynResource};
use ww_link::{UsbFraming, UsbFramingProtocol, UsbSeqFraming};
use ww_tap::Dir;
use ww_tape::FileBlocks;

use crate::test_tape::TestTape;
use crate::{Check, Decoded, Decoder, FrameInfo, MessageInfo, PrefixCounts, Stats, What};

const PACKET: usize = 64;
const MAX_MESSAGE: usize = 1024;

/// The test device: `tap_api::TapDemo` with fixed answers.
mod device {
    use wire_weaver::prelude::*;
    use ww_si::prefix::{Centi, One};
    use ww_si::units::Ratio;
    use ww_si::{AnyQuantity, Fixed, Prefix, SIExpr, Unit};
    use ww_stream::observe::ObserveLimits;

    pub struct Server {
        blob: [u8; 512],
        name: String,
        observe: api_impl::ObserveState,
    }

    impl Server {
        pub fn new() -> Self {
            Server {
                blob: core::array::from_fn(|i| i as u8),
                name: "tapped".into(),
                observe: api_impl::ObserveState::new(ObserveLimits::DEFAULT),
            }
        }

        async fn add(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
            a: u32,
            b: u32,
        ) -> RpcResult<u32> {
            Ready(a.wrapping_add(b))
        }

        async fn blob(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
            len: u16,
        ) -> RpcResult<RefVec<'_, u8>> {
            Ready(RefVec::new_bytes(&self.blob[..usize::from(len).min(512)]))
        }

        async fn get_speed(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
        ) -> GetResult<Fixed<tap_api::Rpm, One, u16>, CommonError<'_>> {
            Value(Fixed::new(7))
        }

        async fn set_speed(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
            _value: Fixed<tap_api::Rpm, One, u16>,
        ) -> SetResult<CommonError<'_>> {
            Set
        }

        async fn get_load(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
        ) -> GetResult<Fixed<Ratio, Centi, u16>, CommonError<'_>> {
            Value(Fixed::new(42))
        }

        async fn get_state(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
        ) -> GetResult<tap_api::State, CommonError<'_>> {
            Value(tap_api::State(3))
        }

        async fn set_user_name(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
            value: MaxLen<&'_ str, 32>,
        ) -> SetResult<CommonError<'_>> {
            self.name = value.to_string();
            Set
        }

        async fn get_user_name(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
        ) -> GetResult<MaxLen<&'_ str, 32>, CommonError<'_>> {
            Value(MaxLen::new_unchecked(&self.name))
        }

        async fn get_sensor(
            &mut self,
            _cx: &mut Context<'_, impl EventOut>,
        ) -> GetResult<AnyQuantity<'_>, CommonError<'_>> {
            let rpm = <tap_api::Rpm as Unit>::EXPR.def().expect("a named unit").id;
            Value(AnyQuantity::new(Prefix::Unit, 1500u16, SIExpr::Ref(rpm)))
        }

        fn observe_state(&mut self) -> &mut api_impl::ObserveState {
            &mut self.observe
        }
    }

    impl api_impl::TapDemoConsts for Server {
        const USER_CAPABILITIES: u8 = 32;
    }

    pub mod api_impl {
        wire_weaver::ww_codegen!(
            tap_api :: TapDemo for super::Server,
            server = true, no_alloc = true, use_async = true,
            method_model = "_=immediate",
            property_model = "_=get_set",
        );
    }

    impl WireWeaverAsyncApiBackend for Server {
        type Medium = ();

        async fn process_bytes<'a>(
            &mut self,
            out: &mut EventWriter<'_, impl MessageSink>,
            medium: (),
            data: &[u8],
            scratch: &'a mut [u8],
        ) -> Result<&'a [u8], ShrinkWrapError> {
            self.process_request_bytes(data, scratch, out, medium).await
        }

        fn version(&self) -> FullVersion<'_> {
            tap_api::TAP_DEMO_FULL_GID
        }
    }
}

/// The device's IN endpoint, dropping one packet when told: `drop_at` counts packets down, the one at 0 is lost and
/// `dropped` notified.
struct LossySink<F> {
    inner: UsbSink<F>,
    drop_at: Arc<AtomicIsize>,
    dropped: Arc<Notify>,
}

impl<F: UsbFramingProtocol> ww_device::PacketSink for LossySink<F> {
    type Error = Disconnected;
    type Framing = F;

    async fn write_packet(&mut self, packet: &[u8]) -> Result<(), Self::Error> {
        if self.drop_at.fetch_sub(1, Ordering::SeqCst) == 0 {
            self.dropped.notify_one();
            return Ok(());
        }
        self.inner.write_packet(packet).await
    }
}

/// A [device::Server] on simulated USB with framing `F` under `path`, in a thread of its own until the returned
/// sender is dropped. Its introspection data goes into the test resolver's cache first, as a client that downloaded
/// it would have it.
fn spawn_device<F: UsbFramingProtocol + 'static>(
    path: &str,
    drop_at: Arc<AtomicIsize>,
    dropped: Arc<Notify>,
) -> oneshot::Sender<()> {
    tests_common::api!(device::api_impl).seed();
    let (sink, source) = usb_device_framed::<F>(path, PACKET);
    let sink = LossySink {
        inner: sink,
        drop_at,
        dropped,
    };
    let (stop_tx, mut stop) = oneshot::channel::<()>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("the device's runtime");
        rt.block_on(async move {
            let config = ww_device::LinkConfig::new(
                tap_api::TAP_DEMO_FULL_GID,
                tap_api::TAP_DEMO_WIRE_VERSION,
                device::api_impl::api_hash(),
                wire_weaver::ww_version::ImageFacts::default(),
                ww_client_server::COMPACT_VERSION,
            );
            let mut tx_frame = [0u8; PACKET];
            let mut rx_buf = ww_device::RxBuffer::<PACKET, MAX_MESSAGE>::new();
            let tx = ww_device::FramedTx::new(sink, &mut tx_frame);
            let rx = ww_device::FramedRx::new(source, rx_buf.assembly_buf(PACKET));
            let (mut scratch, mut event_scratch) = ([0u8; MAX_MESSAGE], [0u8; MAX_MESSAGE]);
            let mut server = ww_device::Server::new(
                config,
                tx,
                rx,
                TokioClock::new(),
                &mut scratch,
                &mut event_scratch,
            );
            let mut backend = device::Server::new();
            loop {
                tokio::select! {
                    ready = server.wait() => {
                        server.handle(ready, &mut backend).await;
                    }
                    _ = &mut stop => break,
                }
            }
        });
    });
    stop_tx
}

/// An API bundle by its hash, as a capture carries it.
type Bundle = (Vec<u8>, wire_weaver_client::ww_self::ApiBundleOwned);

/// A decoder that knows `bundles`, resolving through the clients' resolver: the devices put their introspection data
/// into its cache.
fn decoder(bundles: &[Bundle]) -> Decoder {
    let mut d = Decoder::default().with_resolver(tests_common::resolver());
    for (hash, bundle) in bundles {
        d.add_bundle(hash, bundle.clone());
    }
    d
}

/// Everything `d` decodes after its input ended.
fn drain(mut d: Decoder) -> (Vec<Decoded>, Stats) {
    d.finish();
    let mut out = vec![];
    while let Some(x) = d.next() {
        out.push(x);
    }
    (out, d.stats.take())
}

/// Everything decoded from `blocks`.
fn decode_all<'a>(
    blocks: impl Iterator<Item = ww_tape::Block<'a>>,
    bundles: &[Bundle],
) -> (Vec<Decoded>, Stats) {
    let mut d = decoder(bundles);
    for b in blocks {
        d.feed(&b);
    }
    drain(d)
}

/// The same blocks, their records fed one by one through the decoder's record input as [Decoder::feed] orders them.
fn decode_records<'a>(
    blocks: impl Iterator<Item = ww_tape::Block<'a>>,
    bundles: &[Bundle],
) -> (Vec<Decoded>, Stats) {
    let mut d = decoder(bundles);
    // the tap's tracks by number, as their definitions say; links numbered by identity in the order first seen
    let mut tracks = std::collections::BTreeMap::new();
    let mut identities: Vec<crate::LinkIdentity> = Vec::new();
    for b in blocks {
        let mut plain = vec![];
        let decoded = b.decode(&mut plain).unwrap();
        let mut records: Vec<_> = decoded.records().map(Result::unwrap).collect();
        records.sort_by_key(|r| r.at.map(|at| at.ticks));
        for r in records {
            // definitions (clocks, tracks) are the store's, not records
            let (Some(track), Some(at)) = (r.track.data(), r.at) else {
                if let Ok(ww_tape::Describe::Track(t)) = ww_tape::Describe::from_ww_bytes(r.item) {
                    let (identity, stream) = crate::link_stream_of(&t).unwrap();
                    let link = match identities.iter().position(|i| *i == identity) {
                        Some(i) => i,
                        None => {
                            identities.push(identity);
                            identities.len() - 1
                        }
                    };
                    tracks.insert(t.number, ((link as u32, stream), t.clock));
                }
                continue;
            };
            let ((link, stream), clock) = tracks[&track];
            assert_eq!(at.clock, clock, "a record on its track's clock");
            let t = u64::try_from(at.ticks).unwrap();
            if stream == ww_tap::ids::SESSION {
                d.session(
                    link,
                    t,
                    ww_tap::SessionOwned::from_ww_bytes_owned(r.item).unwrap(),
                );
            } else {
                d.record(link, stream, t, r.item);
            }
        }
    }
    drain(d)
}

fn u16v(v: u16) -> ValueOwned {
    ValueOwned::Numeric(NumericValue::U16(v))
}

fn u32v(v: u32) -> ValueOwned {
    ValueOwned::Numeric(NumericValue::U32(v))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn linkage_split_shared_lost_and_replay() {
    let stats = linkage::<UsbFraming>("tap_decoder/e2e").await;
    let rx = &stats.links[&0].rx;
    assert_eq!(rx.prefix, PrefixCounts::default(), "no prefix to check");
    assert!(!stats.report().contains("frame prefix"));
}

/// The same with the frame counter diagnostic: the lost packet shows as lost by its prefix, nothing else does.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn frame_counter_counts_the_lost_packet() {
    let stats = linkage::<UsbSeqFraming>("tap_decoder/e2e_seq").await;
    let link = &stats.links[&0];
    assert_eq!(link.rx.prefix.lost, 1, "{:?}", link.rx.prefix);
    assert_eq!(link.rx.prefix.dropped(), 0);
    // every frame but the device's hello, which is outside the framer and has no prefix (EVO-23)
    assert_eq!(link.rx.prefix.checked, link.rx.frames - 1);
    assert_eq!((link.tx.prefix.lost, link.tx.prefix.dropped()), (0, 0));
    assert_eq!(link.rx.prefix_bytes, link.rx.frames - 1);
    assert!(stats.report().contains("frame prefix: 1 lost, 0 dropped"));
}

/// Calls over a simulated USB device with framing `F`, one reply packet lost; checks what both framings have in
/// common and returns the stats.
async fn linkage<F: UsbFramingProtocol + 'static>(path: &str) -> Stats {
    let drop_at = Arc::new(AtomicIsize::new(isize::MAX));
    let dropped = Arc::new(Notify::new());
    let _device = spawn_device::<F>(path, drop_at.clone(), dropped.clone());

    let tape = TestTape::new();
    let client = DynClient::from_config(
        ClientConfig::new()
            .in_process_path(path.to_owned())
            .resolver(tests_common::resolver())
            .tap(tape.source()),
    )
    .connect()
    .await
    .expect("connect");
    let introspect = client.device_introspect().expect("the device's API");
    let bundles: Vec<Bundle> = vec![(
        introspect.api_hash.no_docs.hash.clone(),
        (*introspect.sent_api_bundle).clone(),
    )];
    let root = DynResource::root(introspect.api_bundle.clone());
    let (add, blob) = (root.child("add").unwrap(), root.child("blob").unwrap());

    // many small requests at once share host packets; 300 B replies are split over several device packets
    const N: u32 = 20;
    let calls: Vec<_> = (0..N)
        .map(|i| {
            let (add, blob, cmd) = (add.clone(), blob.clone(), client.cmd().clone());
            tokio::spawn(async move {
                let sum = add.call(&cmd, &[u32v(i), u32v(100)]).await.unwrap();
                assert_eq!(sum, Some(u32v(i + 100)));
                let bytes = blob.call(&cmd, &[u16v(300)]).await.unwrap();
                assert!(matches!(bytes, Some(ValueOwned::Vec(v)) if v.len() == 300));
            })
        })
        .collect();
    for c in calls {
        c.await.unwrap();
    }

    // one call alone, the second packet of its reply lost: the reply never comes
    drop_at.store(1, Ordering::SeqCst);
    let lost = {
        let (blob, cmd) = (blob.clone(), client.cmd().clone());
        tokio::spawn(async move { blob.call(&cmd, &[u16v(300)]).await })
    };
    dropped.notified().await;
    // the link goes on
    let after = blob.call(client.cmd(), &[u16v(300)]).await.unwrap();
    assert!(matches!(after, Some(ValueOwned::Vec(v)) if v.len() == 300));
    client.disconnect().asynch().await.unwrap();
    assert!(lost.await.unwrap().is_err(), "reply lost with its packet");
    tape.flush();

    let chunks: Vec<ww_tape::BlockBuf> = tape
        .blocks()
        .into_iter()
        .map(|b| ww_tape::BlockBuf::new(b).unwrap())
        .collect();
    let (live, stats) = decode_all(chunks.iter().map(|c| c.block()), &bundles);
    // the record input gives what the blocks give
    let (by_record, by_record_stats) = decode_records(chunks.iter().map(|c| c.block()), &bundles);
    assert_eq!(format!("{by_record:?}"), format!("{live:?}"));
    assert_eq!(
        by_record_stats,
        Stats {
            blocks: 0,
            ..stats.clone()
        }
    );

    // frame <-> message linkage
    let frames: Vec<&FrameInfo> = live
        .iter()
        .filter_map(|d| match &d.what {
            What::Frame(f) => Some(f),
            _ => None,
        })
        .collect();
    let messages: Vec<&MessageInfo> = live
        .iter()
        .filter_map(|d| match &d.what {
            What::Message(m) => Some(m),
            _ => None,
        })
        .collect();
    let shared = frames
        .iter()
        .find(|f| f.dir == Dir::Tx && f.outcome.completed.len() >= 3)
        .expect("a host packet with several requests");
    let in_shared = messages
        .iter()
        .filter(|m| m.dir == Dir::Tx && m.frames.iter().any(|p| p.frame_no == shared.no))
        .count();
    assert!(in_shared >= 3, "requests link back to their shared packet");
    let split: Vec<&&MessageInfo> = messages
        .iter()
        .filter(|m| m.dir == Dir::Rx && m.frames.len() >= 2)
        .collect();
    assert!(
        split.len() >= N as usize,
        "300 B replies over several packets"
    );
    for m in &split {
        let total: u32 = m.frames.iter().map(|p| p.len).sum();
        assert_eq!(total as usize, m.bytes.len(), "parts add up to the message");
    }
    // cross-check: every message that went through the framer is the one the transport saw
    for m in &messages {
        assert!(
            matches!(m.check, Check::Match | Check::NoFrames),
            "{:?} {:?} no {:?}",
            m.dir,
            m.check,
            m.no
        );
    }
    let linked = messages.iter().filter(|m| m.check == Check::Match).count();
    assert!(linked >= 4 * N as usize, "requests and replies are linked");
    // calls named from the device's API; replies paired with their requests by seq, all but the lost one
    let titled = |title: &str, replies: bool| {
        messages
            .iter()
            .filter(|m| {
                m.api
                    .as_ref()
                    .is_some_and(|a| a.title == title && a.reply_to.is_some() == replies)
            })
            .count()
    };
    assert_eq!(titled("Request Call add", false), N as usize);
    assert_eq!(titled("Event Value add", true), N as usize);
    assert_eq!(titled("Request Call blob", false), N as usize + 2);
    assert_eq!(titled("Event Value blob", true), N as usize + 1);

    // the lost packet: the reply's later pieces don't fit, its partial message is dropped
    let rx = &stats.links[&0].rx;
    assert!(rx.partial_dropped + rx.dropped_pieces >= 1, "{rx:?}");
    assert_eq!(rx.mismatches, 0);
    let tx = &stats.links[&0].tx;
    assert!(
        tx.messages >= 2 * u64::from(N) && tx.frames < tx.messages,
        "{tx:?}"
    );
    assert!(rx.split_messages >= u64::from(N) && rx.split_frames > rx.split_messages);
    assert!(rx.fill_ratio().unwrap() > 0.5);
    assert!(tx.latency_count > 0);
    assert!(stats.report().contains("split over"));

    // a capture is the blocks back to back; read back, it decodes to the same
    let file = tape.file();
    let mut rd = FileBlocks::new(&file[..]);
    let mut blocks = vec![];
    while let Some(b) = rd.next_block() {
        blocks.push(ww_tape::BlockBuf::new(b.unwrap().bytes().to_vec()).unwrap());
    }
    let (replayed, replay_stats) = decode_all(blocks.iter().map(|b| b.block()), &bundles);
    assert_eq!(format!("{replayed:?}"), format!("{live:?}"));
    assert_eq!(replay_stats, stats);
    stats
}

/// Generic `global_fixture_api::GlobalDemo` client: knows nothing of the device's API.
struct GlobalDemoClient {
    cmd: wire_weaver_client::Commander,
}

mod global_demo_client {
    use super::GlobalDemoClient;
    wire_weaver::ww_codegen!(
        global_fixture_api :: GlobalDemo for GlobalDemoClient,
        client = "trait_client",
        no_alloc = false,
        use_async = true,
    );
}

/// API-20: a call to a global trait is named by the device's API on a full connection and by the trait's
/// `ww_global` name on a traits-only one, where it goes out by the trait's `ww_global` id; a capture file decodes the
/// same.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn global_trait_calls_are_named() {
    for traits_only in [false, true] {
        let path = format!("tap_decoder/global_{traits_only}");
        let _device = spawn_device::<UsbFraming>(
            &path,
            Arc::new(AtomicIsize::new(isize::MAX)),
            Arc::new(Notify::new()),
        );
        let tape = TestTape::new();
        let config = ClientConfig::new()
            .in_process_path(path)
            .resolver(tests_common::resolver())
            .tap(tape.source());
        let config = if traits_only {
            config.traits_only()
        } else {
            config
        };
        let client = DynClient::from_config(config)
            .connect()
            .await
            .expect("connect");
        let bundles: Vec<Bundle> = client
            .device_introspect()
            .map(|i| {
                (
                    i.api_hash.no_docs.hash.clone(),
                    (*i.sent_api_bundle).clone(),
                )
            })
            .into_iter()
            .collect();
        assert_eq!(
            bundles.is_empty(),
            traits_only,
            "no introspection traits-only"
        );
        let demo = GlobalDemoClient {
            cmd: client
                .global_attachment(global_fixture_api::GLOBAL_DEMO_FULL_GID, "GlobalDemo")
                .cmd_tx_take(),
        };
        let name = demo.read_name().read().await.unwrap();
        assert_eq!(&*name, "tapped");
        client.disconnect().asynch().await.unwrap();
        tape.flush();

        let chunks: Vec<ww_tape::BlockBuf> = tape
            .blocks()
            .into_iter()
            .map(|b| ww_tape::BlockBuf::new(b).unwrap())
            .collect();
        let (live, _) = decode_all(chunks.iter().map(|c| c.block()), &bundles);
        let resource = match traits_only {
            true => "global_fixture_api::GlobalDemo.name",
            false => "user.name",
        };
        let api = |m: &MessageInfo| m.api.clone();
        let messages: Vec<(MessageInfo, crate::ApiInfo)> = live
            .iter()
            .filter_map(|d| match &d.what {
                What::Message(m) => api(m).map(|a| (m.clone(), a)),
                _ => None,
            })
            .collect();
        let request = messages
            .iter()
            .find(|(_, a)| a.title == format!("Request Read {resource}"))
            .unwrap_or_else(|| panic!("traits_only: {traits_only}, {messages:#?}"));
        assert!(
            messages
                .iter()
                .any(|(_, a)| a.title == format!("Event Value {resource}") && a.reply_to.is_some()),
            "{messages:#?}"
        );
        if traits_only {
            // by the trait's ww_global id (516): 7 bytes, the crate name alone would take 18
            assert_eq!(request.0.bytes.len(), 7, "{:?}", request.0.bytes);
        }

        let file = tape.file();
        let mut rd = FileBlocks::new(&file[..]);
        let mut blocks = vec![];
        while let Some(b) = rd.next_block() {
            blocks.push(ww_tape::BlockBuf::new(b.unwrap().bytes().to_vec()).unwrap());
        }
        let (replayed, _) = decode_all(blocks.iter().map(|b| b.block()), &bundles);
        assert_eq!(format!("{replayed:?}"), format!("{live:?}"));
    }
}

/// Reading an `AnyQuantity` of the API's own `Rpm`, sent by reference: decoded with only the API bundle the device
/// sent (no lookup, no local cache), the unit found in that bundle, not in anything compiled into the reader.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_quantity_by_reference_decodes_from_the_sent_bundle() {
    use wire_weaver_client::quantity::{DynQuantity, DynUnit};
    use ww_si::Unit;

    let path = "tap_decoder/quantity";
    let _device = spawn_device::<UsbFraming>(
        path,
        Arc::new(AtomicIsize::new(isize::MAX)),
        Arc::new(Notify::new()),
    );
    let tape = TestTape::new();
    let client = DynClient::from_config(
        ClientConfig::new()
            .in_process_path(path.to_owned())
            .resolver(tests_common::resolver())
            .tap(tape.source()),
    )
    .connect()
    .await
    .expect("connect");
    let introspect = client.device_introspect().unwrap();
    let bundles: Vec<Bundle> = vec![(
        introspect.api_hash.no_docs.hash.clone(),
        (*introspect.sent_api_bundle).clone(),
    )];
    let root = DynResource::root(introspect.api_bundle.clone());
    root.child("sensor")
        .unwrap()
        .read(client.cmd())
        .await
        .unwrap();
    client.disconnect().asynch().await.unwrap();
    tape.flush();

    // snapshots of this workspace's dependencies only, no cache: the sent bundle is all it has of the device's API
    let mut decoder = Decoder::default().with_resolver(crate::test_resolver());
    for (hash, bundle) in &bundles {
        decoder.add_bundle(hash, bundle.clone());
    }
    for b in tape.blocks() {
        decoder.feed(&ww_tape::BlockBuf::new(b).unwrap().block());
    }
    decoder.finish();
    let mut quantities = vec![];
    while let Some(d) = decoder.next() {
        let What::Message(m) = &d.what else {
            continue;
        };
        let Some(envelope) = m.api.as_ref().and_then(|a| a.envelope.clone()) else {
            continue;
        };
        let bundle = decoder.link_api(d.link.unwrap()).unwrap().bundle().clone();
        let (Ok(values), Some(ty)) = (&envelope.explained.result, envelope.types.first()) else {
            continue;
        };
        if let Some(q) = values
            .first()
            .and_then(|v| DynQuantity::from_value(v, ty, &bundle))
        {
            quantities.push(q);
        }
    }
    let [q] = quantities.as_slice() else {
        panic!("one quantity in the capture: {quantities:?}");
    };
    let rpm: ww_si::exact_units::UnitDefOwned =
        (*<tap_api::Rpm as Unit>::EXPR.def().expect("a named unit")).into();
    assert_eq!(q.unit, DynUnit::Named(rpm));
    assert_eq!(q.to_string(), "1500 rpm");
    assert!((q.to_si_f64().unwrap() - 50.0 * std::f64::consts::PI).abs() < 1e-9);
}
