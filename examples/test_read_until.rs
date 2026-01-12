use std::{env, io};

fn main() -> io::Result<()> {
    let mut args: Vec<_> = env::args_os().skip(1).collect();
    if args.is_empty() {
        args.push("-".into());
    }

    let mut diamond = diamond_op::new();
    assert!(diamond.current_arg().is_none());

    let mut i = 0;
    let mut buf = Vec::new();
    while diamond.read_until(b'\n', &mut buf)? != 0 {
        print!("[{}] {}", i, String::from_utf8_lossy(&buf));

        let arg = diamond.current_arg().unwrap();
        let pos = args.iter().position(|e| e == arg).unwrap();
        args.drain(..pos);

        buf.clear();
        i += 1;
    }

    assert!(diamond.current_arg().is_none());
    Ok(())
}
