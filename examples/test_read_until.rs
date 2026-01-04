use std::io;

fn main() -> io::Result<()> {
    let mut i = 0;
    let mut buf = Vec::new();
    let mut diamond = diamond_op::new();
    while diamond.read_until(b'\n', &mut buf)? != 0 {
        print!("[{}] {}", i, String::from_utf8_lossy(&buf));
        buf.clear();
        i += 1;
    }
    Ok(())
}
