use std::fmt::{self, Display};
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Empty,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            Io(err) => err.fmt(f),
            Empty => f.write_str("no source files found"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
