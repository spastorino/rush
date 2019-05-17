use std::convert::TryFrom;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::iter::Iterator;
use std::process::Command;
use std::str::SplitWhitespace;
use std::vec::IntoIter;

pub enum Expression<'a> {
    Cmd(Cmd<'a>),
    Compound(Box<Compound<'a>>),
}

#[derive(Debug)]
pub struct Cmd<'a> {
    pub binary: &'a OsStr,
    pub args: LineIter<'a>,
}

pub struct Compound<'a> {
    pub op: Op,
    pub left: Expression<'a>,
    pub right: Expression<'a>,
}

pub enum Op {
    Semicolon,
}

#[derive(Debug)]
pub struct LineIter<'a>(SplitWhitespace<'a>);

#[derive(Debug)]
pub enum Error {
    EmptyLine,
    Io(io::Error),
}

impl<'a> TryFrom<&'a str> for Expression<'a> {
    type Error = Error;

    // Extract the expression from the commandline
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let mut cmds = vec![];

        for cmd in line.split(';') {
            cmds.push(Cmd::try_from(cmd)?);
        }

        Ok(Expression::build_semicolon_expression(cmds.into_iter()))
    }
}

impl<'a> Expression<'a> {
    pub fn run(self) -> Result<bool, Error> {
        match self {
            Expression::Cmd(cmd) => cmd.run(),

            Expression::Compound(compound) => match compound.op {
                Op::Semicolon => {
                    compound.left.run()?;
                    compound.right.run()
                }
            },
        }
    }

    fn build_semicolon_expression(mut cmds: IntoIter<Cmd<'a>>) -> Expression<'a> {
        assert!(cmds.len() >= 1);
        let cmd_left = cmds.next().unwrap();

        if cmds.len() == 0 {
            Expression::Cmd(cmd_left)
        } else {
            Expression::Compound(Box::new(Compound {
                op: Op::Semicolon,
                left: Expression::Cmd(cmd_left),
                right: Expression::build_semicolon_expression(cmds),
            }))
        }
    }
}

impl<'a> Cmd<'a> {
    pub fn run(self) -> Result<bool, Error> {
        match Command::new(&self.binary).args(self.args).spawn() {
            Ok(mut child) => child
                .wait()
                .map(|exit_status| exit_status.success())
                .map_err(|e| Error::Io(e)),
            Err(_) => io::stderr()
                .write(b"Command not found\n")
                .map(|_| true)
                .map_err(|e| Error::Io(e)),
        }
    }
}

impl<'a> TryFrom<&'a str> for Cmd<'a> {
    type Error = Error;

    // Extract the command and its arguments from the commandline
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let mut args = LineIter::from(line);
        let binary = args.next().map(OsStr::new).ok_or(Error::EmptyLine)?;

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
        match Cmd::try_from("") {
            Err(Error::EmptyLine) => assert!(true),
            _ => assert!(false),
        }
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

    #[test]
    fn test_semicolon_expression() {
        match Expression::try_from("echo 1 2 3; ls").unwrap() {
            Expression::Compound(compound) => match *compound {
                Compound {
                    op: Op::Semicolon,
                    left:
                        Expression::Cmd(Cmd {
                            binary: binary_left,
                            args: mut args_left,
                        }),

                    right:
                        Expression::Cmd(Cmd {
                            binary: binary_right,
                            args: mut args_right,
                        }),
                } => {
                    assert_eq!(binary_left, OsStr::new("echo"));
                    assert_eq!(args_left.next(), Some("1"));
                    assert_eq!(args_left.next(), Some("2"));
                    assert_eq!(args_left.next(), Some("3"));
                    assert_eq!(args_left.next(), None);

                    assert_eq!(binary_right, OsStr::new("ls"));
                    assert_eq!(args_right.next(), None);
                }

                _ => assert!(false),
            },

            _ => assert!(false),
        }
    }
}
