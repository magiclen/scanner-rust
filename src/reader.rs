use std::io::{self, ErrorKind, Read};

#[inline]
pub(crate) fn read<R: Read>(reader: &mut R, buffer: &mut [u8]) -> io::Result<usize> {
    loop {
        match reader.read(buffer) {
            Ok(size) if size <= buffer.len() => return Ok(size),
            Ok(_) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "reader returned more bytes than the buffer can hold",
                ));
            },
            Err(err) if err.kind() == ErrorKind::Interrupted => (),
            Err(err) => return Err(err),
        }
    }
}
