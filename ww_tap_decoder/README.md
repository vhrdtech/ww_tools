# ww_tap_decoder

Decoder of [WireWeaver](https://github.com/vhrdtech/wire_weaver) link tap captures. A client's link reports what
crosses it to a `Tap` (`wire_weaver_client::tap::TapSource`); a capture tool writes that as `ww_tape` blocks on the
tracks this crate names (`tape`). The `Decoder` reads the blocks, or the records one by one, back as frames,
messages and API calls:

- frames linked to the messages they carry, checked against the transport's message records;
- requests paired with their replies by seq, named and explained from the API bundle the capture carries or from
  snapshots (global traits by their `ww_global` id);
- framer and link statistics (`Stats`): split and shared frames, fill ratio, losses, latency.
