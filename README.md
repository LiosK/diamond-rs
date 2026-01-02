# Perl-like diamond operator for Rust

[![Crates.io](https://img.shields.io/crates/v/diamond_op)](https://crates.io/crates/diamond_op)
[![License](https://img.shields.io/crates/l/diamond_op)](https://github.com/LiosK/diamond-rs/blob/main/LICENSE)

```rust
// Prints all lines from files and standard input specified by command line
// arguments or from standard input if no argument is given.
fn main() {
    for line in diamond_op::new().line_iter() {
        print!("{}", line.expect("failed to read line"));
    }
}
```

```bash
# Prints all lines from file1.txt, file2.txt, standard input, and file3.txt.
mycmd file1.txt file2.txt - file3.txt
```

## License

Licensed under the Apache License, Version 2.0.
