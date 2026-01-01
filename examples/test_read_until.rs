use std::{io, str};

use diamond_op::Diamond;

fn main() -> io::Result<()> {
    let mut i = 0;
    let mut buf = Vec::new();
    let mut diamond = Diamond::default();
    while diamond.read_until(b'\n', &mut buf)? != 0 {
        print!("[{}] {}", i, as_str(&buf)?);
        buf.clear();
        i += 1;
    }
    Ok(())
}

fn as_str(buf: &[u8]) -> io::Result<&str> {
    str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
