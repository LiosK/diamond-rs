use std::io;

use diamond_op::Diamond;

fn main() -> io::Result<()> {
    let mut i = 0;
    let mut buf = String::new();
    let mut diamond = Diamond::default();
    while diamond.read_line(&mut buf)? != 0 {
        print!("[{}] {}", i, buf);
        buf.clear();
        i += 1;
    }
    Ok(())
}
