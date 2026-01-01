use std::{io, io::Read as _};

fn main() -> io::Result<()> {
    let mut buf = String::new();
    diamond_op::new().reader().read_to_string(&mut buf)?;
    print!("{}", buf);
    Ok(())
}
