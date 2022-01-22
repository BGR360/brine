use std::io;

/// The [`minecraft_varint`] uses the wrong encoding for signed VarInts, so this
/// is a workaround.
pub trait VarIntRead {
    fn read_var_i32(&mut self) -> io::Result<i32>;
    fn read_var_i64(&mut self) -> io::Result<i64>;
}

impl<R: io::Read> VarIntRead for R {
    fn read_var_i32(&mut self) -> io::Result<i32> {
        minecraft_varint::VarIntRead::read_var_u32(self).map(|v| v.try_into().unwrap())
    }

    fn read_var_i64(&mut self) -> io::Result<i64> {
        minecraft_varint::VarIntRead::read_var_u64(self).map(|v| v.try_into().unwrap())
    }
}
