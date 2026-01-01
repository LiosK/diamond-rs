use std::io;

fn main() -> io::Result<()> {
    for (i, line) in diamond_op::new().line_iter().enumerate() {
        print!("[{}] {}", i, line?);
    }
    Ok(())
}
