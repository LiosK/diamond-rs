use std::{io, io::Read as _};

use diamond_op::Diamond;

fn main() -> io::Result<()> {
    let mut buf = String::new();
    Diamond::default().reader().read_to_string(&mut buf)?;
    print!("{}", buf);
    Ok(())
}
