mod cmd;

use std::convert::TryFrom;
use std::io::{self, Write};
use std::process::Command;

use self::cmd::{Cmd, ParseError};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        stdout.write(b"> ")?;
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        match Cmd::try_from(input.as_ref()) {
            Ok(cmd) => match Command::new(cmd.binary).args(cmd.args).spawn() {
                Ok(mut child) => {
                    child.wait()?;
                }
                Err(_) => {
                    stderr.write(b"Command not found\n")?;
                }
            },

            Err(ParseError::EmptyLine) => {}
        }
    }
}
