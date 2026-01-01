use std::io;

use diamond_op::Diamond;

fn main() -> io::Result<()> {
    for (i, line) in Diamond::default().line_iter().enumerate() {
        print!("[{}] {}", i, line?);
    }
    Ok(())
}
