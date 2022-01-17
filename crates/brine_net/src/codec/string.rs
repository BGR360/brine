use std::{mem, str::Utf8Error};

use async_codec::{Decode, DecodeResult, Encode, EncodeResult};
use bevy::log;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

/// A simple codec that sends and receives length-prefixed strings.
#[derive(Debug, Default, Clone)]
pub struct StringCodec;

impl Encode for StringCodec {
    type Item = String;
    type Error = ();

    fn encode(&mut self, item: &Self::Item, mut buf: &mut [u8]) -> EncodeResult<Self::Error> {
        let bytes_needed = item.as_bytes().len() + mem::size_of::<u32>();
        if buf.len() < bytes_needed {
            return EncodeResult::Overflow(bytes_needed);
        }

        buf.write_u32::<BigEndian>(item.as_bytes().len().try_into().unwrap())
            .unwrap();
        buf[..item.as_bytes().len()].copy_from_slice(item.as_bytes());

        EncodeResult::Ok(bytes_needed)
    }
}

impl Decode for StringCodec {
    type Item = String;
    type Error = Utf8Error;

    fn decode(&mut self, buf: &mut [u8]) -> (usize, DecodeResult<Self::Item, Self::Error>) {
        log::trace!("decode: buf = {:?}", &buf);

        let mut buf: &[u8] = buf;
        if buf.len() < mem::size_of::<u32>() {
            return (0, DecodeResult::UnexpectedEnd);
        }
        let len = buf.read_u32::<BigEndian>().unwrap() as usize;

        log::trace!("decode: len={}, buf={:?}", len, &buf);

        if buf.len() < len {
            return (0, DecodeResult::UnexpectedEnd);
        }
        let string_bytes = &buf[..len];
        (
            mem::size_of::<u32>() + len,
            std::str::from_utf8(string_bytes).map(String::from).into(),
        )
    }
}
