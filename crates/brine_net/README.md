# brine_net

This is a generic library providing an easy(ish) way for Bevy apps to implement
a known protocol over TCP.

The design of this library centers heavily on the
[`async_codec`](https://docs.rs/async-codec/latest/async_codec/) crate. Any
user protocol that implements `async_code::Encode` and `async_codec::Decode`
can be used with the `NetworkPlugin` provided by this crate.

## Examples and Documentation

The rustdocs in [`lib.rs`](src/lib.rs) and [`plugin.rs`](src/plugin.rs) provide
a good description of how the crate is used. The [`examples`](examples/)
directory contains a couple simple examples.

For a much more fully-fledged example, see
[`brine_proto_backend`](../brine_proto_backend/src/backend_stevenarella/codec.rs).

## Limitations

I haven't spent much more time on this library so far beyond that which was
required to get me up and running with a working Minecraft protocol backend. So
there are still some major limitations:

* Only one connection can be active at a time for a given
  `NetworkPlugin<Codec>`.
* Only serverbound connections can be established; there is no way to set up a
  listening socket and implement the server side of things.

Also, `async_codec` does not appear to be without its problems. I've already had
to work around one or two.