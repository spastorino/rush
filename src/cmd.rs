use std::convert::TryFrom;
use std::ffi::OsStr;
use std::str::SplitWhitespace;
use std::iter::Iterator;

#[derive(Debug)]
pub struct Cmd<'a> {
    pub binary: &'a OsStr,
    pub args: LineIter<'a>,
}

#[derive(Debug)]
pub struct LineIter<'a>(SplitWhitespace<'a>);

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    EmptyLine,
}

impl<'a> TryFrom<&'a str> for Cmd<'a> {
    type Error = ParseError;

    // Extract the command and its arguments from the commandline
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let mut args = LineIter::from(line);
        let binary = args.next().map(OsStr::new).ok_or(ParseError::EmptyLine)?;

        Ok(Cmd { binary, args })
    }
}

impl<'a> LineIter<'a> {
    fn from(line: &'a str) -> LineIter {
        LineIter(line.split_whitespace())
    }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_line() {
        assert_eq!(Cmd::try_from("").unwrap_err(), ParseError::EmptyLine);
    }

    #[test]
    fn test_single_binary() {
        let mut cmd = Cmd::try_from("echo").unwrap();

        assert_eq!(cmd.binary, OsStr::new("echo"));
        assert_eq!(cmd.args.next(), None);
    }

    #[test]
    fn test_binary_with_arguments() {
        let cmd = Cmd::try_from("echo 1 2 3").unwrap();

        assert_eq!(cmd.binary, OsStr::new("echo"));
        assert_eq!(cmd.args.collect::<Vec<_>>(), vec!["1", "2", "3"]);
    }
}
