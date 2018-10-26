use std::io;
use std::num;
use std::result;

use tokio::prelude::*;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(num::ParseIntError),
    ProtocolError(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::Parse(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::ProtocolError(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &'a str) -> Error {
        Error::ProtocolError(err.to_string())
    }
}

pub type Result<T> = result::Result<T, Error>;

// impl<T> Future for Result<T> {
//     type Item = T;
//     type Error = Error;
//     fn poll(&mut self) -> Result<Async<T>> {
//         match self {
//             Ok(v) => Ok(Async::Ready(v)),
//             x => x,
//         }
//     }
// }
