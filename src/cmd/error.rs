use std::io;

#[derive(Debug)]
pub enum Error {
    EmptyLine,
    Io(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::EmptyLine => Ok(()),
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::EmptyLine => None,
            Error::Io(e) => Some(e),
        }
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
