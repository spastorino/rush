use std::convert::TryFrom;
use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::iter::Iterator;
use std::path::Path;
use std::process::{self, Command};
use std::str::SplitWhitespace;
use std::vec::IntoIter;

pub enum Expression<'a> {
    Cmd(Cmd<'a>),
    Compound(Box<Compound<'a>>),
}

#[derive(Debug)]
pub enum Cmd<'a> {
    // An invokable command consists of a binary and its arguments
    Invoke(Invoke<'a>),

    Builtin(Builtin<'a>),
}

#[derive(Debug)]
pub struct Invoke<'a> {
    pub binary: &'a OsStr,
    pub args: LineIter<'a>,
}

#[derive(Debug)]
pub enum Builtin<'a> {
    Exit(i32),
    Cd(&'a Path),
}

pub struct Compound<'a> {
    pub op: Op,
    pub left: Expression<'a>,
    pub right: Expression<'a>,
}

pub enum Op {
    Semicolon,
    And,
}

#[derive(Debug)]
pub struct LineIter<'a>(SplitWhitespace<'a>);

#[derive(Debug)]
pub enum Error {
    EmptyLine,
    Io(io::Error),
    NoCmd,
    NoDir,
}

impl<'a> TryFrom<&'a str> for Expression<'a> {
    type Error = Error;

    // Extract the expression from the commandline
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let mut stmts = vec![];

        for stmt in line.split(';') {
            let mut cmds = vec![];

            for cmd in stmt.split("&&") {
                cmds.push(Cmd::try_from(cmd)?);
            }

            stmts.push(Self::build_and_expression(cmds.into_iter()));
        }

        Ok(Self::build_semicolon_expression(stmts.into_iter()))
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

                Op::And => Ok(compound.left.run()? && compound.right.run()?),
            },
        }
    }

    fn build_and_expression(mut cmds: IntoIter<Cmd<'a>>) -> Self {
        let cmd_left = cmds.next().unwrap();

        if cmds.len() == 0 {
            Expression::Cmd(cmd_left)
        } else {
            Expression::Compound(Box::new(Compound {
                op: Op::And,
                left: Expression::Cmd(cmd_left),
                right: Expression::build_and_expression(cmds),
            }))
        }
    }

    fn build_semicolon_expression(mut exprs: IntoIter<Self>) -> Self {
        assert!(exprs.len() >= 1);
        let expr_left = exprs.next().unwrap();

        if exprs.len() == 0 {
            expr_left
        } else {
            Expression::Compound(Box::new(Compound {
                op: Op::Semicolon,
                left: expr_left,
                right: Expression::build_semicolon_expression(exprs),
            }))
        }
    }
}

impl<'a> Cmd<'a> {
    pub fn run(self) -> Result<bool, Error> {
        match self {
            Cmd::Builtin(Builtin::Exit(status)) => {
                process::exit(status);
            }

            Cmd::Builtin(Builtin::Cd(path)) => match path.canonicalize() {
                Ok(path) => env::set_current_dir(&path)
                    .map(|_| true)
                    .map_err(|e| Error::Io(e)),

                Err(e) => Err(Error::Io(e)),
            },

            Cmd::Invoke(Invoke { binary, args }) => match Command::new(binary).args(args).spawn() {
                Ok(mut child) => child
                    .wait()
                    .map(|exit_status| exit_status.success())
                    .map_err(|e| Error::Io(e)),
                Err(_) => io::stderr()
                    .write(b"Command not found\n")
                    .map(|_| true)
                    .map_err(|e| Error::Io(e)),
            },
        }
    }
}

impl<'a> TryFrom<&'a str> for Cmd<'a> {
    type Error = Error;

    // Extract the command and its arguments from the commandline
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let mut args = LineIter::from(line);
        let binary = args.next().map(OsStr::new).ok_or(Error::EmptyLine)?;

        match binary.to_str() {
            Some("exit") => Ok(Cmd::Builtin(Builtin::Exit(0))),

            Some("cd") => {
                let path = args.next().map(OsStr::new).ok_or(Error::NoDir)?;
                Ok(Cmd::Builtin(Builtin::Cd(Path::new(path))))
            }

            Some(_) => Ok(Cmd::Invoke(Invoke { binary, args })),

            _ => Err(Error::NoCmd),
        }
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
        let cmd = Cmd::try_from("echo").unwrap();

        if let Cmd::Invoke(Invoke { binary, mut args }) = cmd {
            assert_eq!(binary, OsStr::new("echo"));
            assert_eq!(args.next(), None);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_binary_with_arguments() {
        let cmd = Cmd::try_from("echo 1 2 3").unwrap();

        if let Cmd::Invoke(Invoke { binary, args }) = cmd {
            assert_eq!(binary, OsStr::new("echo"));
            assert_eq!(args.collect::<Vec<_>>(), vec!["1", "2", "3"]);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_exit_builtin() {
        let cmd = Cmd::try_from("exit").unwrap();

        if let Cmd::Builtin(Builtin::Exit(status)) = cmd {
            assert_eq!(status, 0);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_cd_builtin() {
        let cmd = Cmd::try_from("cd /home").unwrap();

        if let Cmd::Builtin(Builtin::Cd(path)) = cmd {
            assert_eq!(path.to_str(), Some("/home"));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_semicolon_expression() {
        match Expression::try_from("echo 1 2 3; ls").unwrap() {
            Expression::Compound(compound) => match *compound {
                Compound {
                    op: Op::Semicolon,
                    left:
                        Expression::Cmd(Cmd::Invoke(Invoke {
                            binary: binary_left,
                            args: mut args_left,
                        })),

                    right:
                        Expression::Cmd(Cmd::Invoke(Invoke {
                            binary: binary_right,
                            args: mut args_right,
                        })),
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

    #[test]
    fn test_and_expression() {
        match Expression::try_from("echo 1 2 3 && ls").unwrap() {
            Expression::Compound(compound) => match *compound {
                Compound {
                    op: Op::And,
                    left:
                        Expression::Cmd(Cmd::Invoke(Invoke {
                            binary: binary_left,
                            args: mut args_left,
                        })),

                    right:
                        Expression::Cmd(Cmd::Invoke(Invoke {
                            binary: binary_right,
                            args: mut args_right,
                        })),
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
