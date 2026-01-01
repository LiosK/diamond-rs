use std::{io, io::BufRead as _};

use diamond_op::Diamond;

fn main() -> io::Result<()> {
    let mut i = 0;
    let mut buf = String::new();
    let mut reader = io::BufReader::new(Diamond::default().reader());
    while reader.read_line(&mut buf)? != 0 {
        print!("[{}] {}", i, buf);
        buf.clear();
        i += 1;
    }
    Ok(())
}
