//! Perl-like diamond operator for Rust
//!
//! ```rust
//! // Prints all lines from files and standard input specified by command line
//! // arguments or from standard input if no argument is given.
//! fn main() {
//!     for line in diamond_op::new().line_iter() {
//!         print!("{}", line.expect("failed to read line"));
//!     }
//! }
//! ```
//!
//! ```bash
//! # Prints all lines from file1.txt, file2.txt, standard input, and file3.txt.
//! mycmd file1.txt file2.txt - file3.txt
//! ```

use std::io::{self, BufRead};
use std::{env, ffi, fs, iter, slice};

/// Returns a diamond operator instance.
///
/// See the [crate documentation](crate) or [`Diamond`] for usage examples.
pub fn new() -> Diamond {
    Diamond::default()
}

/// A structure that reads lines, like Perl's diamond (`<>`) operator and many Unix filter programs,
/// from files and standard input ("-") specified by command line arguments or from standard input
/// if no argument is given.
#[derive(Debug, Default)]
pub struct Diamond {
    inner: DiamondInner<Reader, Readers<Args>>,
}

impl Diamond {
    /// Reads all bytes into `buf` until the delimiter `byte` or EOF is reached.
    ///
    /// This function works in the same way as [`BufRead::read_until`], except that it also returns
    /// at the EOF of each file or standard input that does not end with the `byte`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buf = Vec::new();
    /// let mut diamond = diamond_op::new();
    /// while diamond.read_until(b'\n', &mut buf)? != 0 {
    ///     print!("{}", String::from_utf8_lossy(&buf));
    ///     buf.clear();
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    /// Reads all bytes into `buf` until a newline (the `0xA` byte) or EOF is reached.
    ///
    /// This function works in the same way as [`BufRead::read_line`], except that it also returns
    /// at the EOF of each file or standard input that does not end with a newline byte.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut buf = String::new();
    /// let mut diamond = diamond_op::new();
    /// while diamond.read_line(&mut buf)? != 0 {
    ///     print!("{}", buf);
    ///     buf.clear();
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }

    /// Returns an iterator over the lines of all files and standard input.
    ///
    /// The returned iterator essentially calls [`read_line`](Self::read_line) on a new `String`
    /// buffer for each iteration and yields it as is. Accordingly, it is different from the
    /// iterator returned from [`BufRead::lines`] in the following points:
    ///
    /// - It also returns at the EOF of each file or standard input that does not end with a
    ///   newline byte.
    /// - It does not strip the newline byte from the end of each line.
    ///
    /// # Examples
    ///
    /// ```rust
    /// for line in diamond_op::new().line_iter() {
    ///     print!("{}", line?);
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn line_iter(self) -> impl Iterator<Item = io::Result<String>> {
        self.inner.line_iter()
    }

    /// Returns a reader that reads bytes as a single stream.
    ///
    /// The returned reader reads bytes, treating all files and standard input as a consolidated
    /// single stream and ignoring the EOF of each file or standard input in between, which is
    /// different from the behavior of other methods in this type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Read as _;
    /// let mut buf = String::new();
    /// diamond_op::new().reader().read_to_string(&mut buf)?;
    /// print!("{}", buf);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn reader(self) -> impl BufRead {
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
    R: BufRead,
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

    fn reader(self) -> impl BufRead {
        struct SingleStreamReader<R, I>(DiamondInner<R, I>);

        impl<R, I> io::Read for SingleStreamReader<R, I>
        where
            R: BufRead,
            I: Iterator<Item = io::Result<R>>,
        {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let n = self.fill_buf()?.read(buf)?;
                self.consume(n);
                Ok(n)
            }
        }

        impl<R, I> BufRead for SingleStreamReader<R, I>
        where
            R: BufRead,
            I: Iterator<Item = io::Result<R>>,
        {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                loop {
                    if let Some(reader) = &mut self.0.current {
                        let ret = reader.fill_buf()?;
                        if !ret.is_empty() {
                            // Intends to `return Ok(ret);` but hacks the borrow checker to work
                            // around the "conditional returns" limitation:
                            // https://github.com/rust-lang/rust/issues/51545
                            return Ok(unsafe { slice::from_raw_parts(ret.as_ptr(), ret.len()) });
                        }
                        self.0.current = None;
                    } else if let Some(reader) = self.0.remaining.next() {
                        self.0.current = Some(reader?);
                    } else {
                        return Ok(&[]);
                    }
                }
            }

            fn consume(&mut self, amount: usize) {
                if let Some(reader) = &mut self.0.current {
                    reader.consume(amount);
                }
            }
        }

        SingleStreamReader(self)
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
struct Args(Option<iter::Fuse<env::ArgsOs>>);

impl Iterator for Args {
    type Item = ffi::OsString;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(args) = &mut self.0 {
            args.next()
        } else {
            let mut args = env::args_os().fuse();
            args.next(); // skip program name
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
    type Item = io::Result<Reader>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|arg| Reader::open(arg.as_ref()))
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum Reader {
    Stdin(io::StdinLock<'static>),
    File(io::BufReader<fs::File>),
}

impl Reader {
    fn open(arg: &ffi::OsStr) -> io::Result<Self> {
        if arg == "-" {
            Ok(Self::Stdin(io::stdin().lock()))
        } else {
            let file = fs::File::open(arg)?;
            Ok(Self::File(io::BufReader::new(file)))
        }
    }

    #[inline]
    fn as_buf_read_mut(&mut self) -> &mut dyn BufRead {
        match self {
            Self::Stdin(r) => r,
            Self::File(r) => r,
        }
    }
}

impl io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_buf_read_mut().read(buf)
    }
}

impl BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.as_buf_read_mut().fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.as_buf_read_mut().consume(amount)
    }
}
