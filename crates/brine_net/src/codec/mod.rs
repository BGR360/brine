//! Implementations of a small number of network codecs.

mod dummy;
mod string;

pub use dummy::DummyCodec;
pub use string::StringCodec;
