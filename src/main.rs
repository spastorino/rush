use std::io::{self, Write};
use std::process::Command;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        stdout.write(b"> ")?;
        stdout.flush()?;
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let mut args = input.split_whitespace();

        if let Some(cmd) = args.next() {
            match Command::new(cmd).args(args).spawn() {
                Ok(mut child) => {
                    child.wait()?;
                }
                Err(_) => {
                    stderr.write(b"Command not found\n")?;
                }
            }
        }
    }
}
