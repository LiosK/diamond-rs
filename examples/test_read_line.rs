use std::io;

fn main() -> io::Result<()> {
    let mut i = 0;
    let mut buf = String::new();
    let mut diamond = diamond_op::new();
    while diamond.read_line(&mut buf)? != 0 {
        print!("[{}] {}", i, buf);
        buf.clear();
        i += 1;
    }
    Ok(())
}
