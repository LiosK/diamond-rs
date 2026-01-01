//! Perl-like diamond operator for Rust
//!
//! ```rust
//! for line in diamond_op::new().line_iter() {
//!     print!("{}", line?);
//! }
//! # Ok::<(), std::io::Error>(())
//! ```

use std::{env, ffi, fs, io, iter};

/// Returns a diamond operator instance.
pub fn new() -> Diamond {
    Diamond::default()
}

/// A structure that reads lines from multiple files or standard input like Perl's diamond (`<>`)
/// operator.
#[derive(Default)]
pub struct Diamond {
    inner: DiamondInner<Box<dyn io::BufRead>, Readers<Args>>,
}

impl Diamond {
    /// Reads all bytes into `buf` until the delimiter `byte` or EOF is reached.
    pub fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    /// Reads all bytes into `buf` until a newline (the `0xA` byte) or EOF is reached.
    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }

    /// Returns an iterator over the lines of all files or standard input.
    pub fn line_iter(self) -> impl Iterator<Item = io::Result<String>> {
        self.inner.line_iter()
    }

    /// Returns a reader that reads bytes as a single stream.
    pub fn reader(self) -> impl io::Read {
        self.inner.reader()
    }
}

/// The inner structure separated for easier testing and to internal type hiding.
#[derive(Debug)]
struct DiamondInner<R, I> {
    current: Option<R>,
    remaining: I,
}

impl<R, I> DiamondInner<R, I>
where
    R: io::BufRead,
    I: Iterator<Item = io::Result<R>>,
{
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.read_inner(|reader| reader.read_until(byte, buf))
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.read_inner(|reader| reader.read_line(buf))
    }

    fn line_iter(mut self) -> impl Iterator<Item = io::Result<String>> {
        iter::from_fn(move || {
            let mut buf = String::new();
            match self.read_line(&mut buf) {
                Ok(0) => None,
                Ok(_) => Some(Ok(buf)),
                Err(e) => Some(Err(e)),
            }
        })
    }

    fn reader(self) -> impl io::Read {
        struct Reader<R, I>(DiamondInner<R, I>);

        impl<R, I> io::Read for Reader<R, I>
        where
            R: io::BufRead,
            I: Iterator<Item = io::Result<R>>,
        {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.0.read_inner(|reader| reader.read(buf))
            }
        }

        Reader(self)
    }

    fn read_inner(&mut self, mut f: impl FnMut(&mut R) -> io::Result<usize>) -> io::Result<usize> {
        loop {
            if let Some(reader) = &mut self.current {
                let ret = f(reader)?;
                if ret != 0 {
                    return Ok(ret);
                }
                self.current = None;
            } else if let Some(reader) = self.remaining.next() {
                self.current = Some(reader?);
            } else {
                return Ok(0);
            }
        }
    }
}

impl<R, I: Default> Default for DiamondInner<R, I> {
    fn default() -> Self {
        Self {
            current: None,
            remaining: I::default(),
        }
    }
}

/// A command line argument iterator that returns "-" if none is given.
#[derive(Debug, Default)]
struct Args(Option<iter::Skip<iter::Fuse<env::ArgsOs>>>);

impl Iterator for Args {
    type Item = ffi::OsString;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(args) = &mut self.0 {
            args.next()
        } else {
            let args = env::args_os().fuse().skip(1);
            self.0.insert(args).next().or_else(|| Some("-".into()))
        }
    }
}

/// An iterator transformer that yields buffered readers from command line arguments.
#[derive(Debug, Default)]
struct Readers<T>(T);

impl<T, U> Iterator for Readers<T>
where
    T: Iterator<Item = U>,
    U: AsRef<ffi::OsStr>,
{
    type Item = io::Result<Box<dyn io::BufRead>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|arg| {
            let arg = arg.as_ref();
            if arg == "-" {
                Ok(Box::new(io::stdin().lock()) as Box<dyn io::BufRead>)
            } else {
                let file = fs::File::open(arg)?;
                Ok(Box::new(io::BufReader::new(file)) as Box<dyn io::BufRead>)
            }
        })
    }
}
