use async_codec::{Decode, DecodeResult, Encode, EncodeResult};

/// A dummy codec useful for testing.
#[derive(Debug, Default, Clone)]
pub struct DummyCodec;

impl Encode for DummyCodec {
    type Item = ();
    type Error = ();

    fn encode(&mut self, _item: &Self::Item, _buf: &mut [u8]) -> EncodeResult<Self::Error> {
        EncodeResult::Ok(0)
    }
}

impl Decode for DummyCodec {
    type Item = ();
    type Error = ();

    fn decode(&mut self, _buffer: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        (0, DecodeResult::Ok(()))
    }
}
