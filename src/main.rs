use std::io::{self, Write};
use std::process::Command;
use std::str::SplitWhitespace;

struct Cmd<'a> {
    comm: Option<&'a str>,
    args: SplitWhitespace<'a>,
}

impl<'a> Cmd<'a> {
    fn parse(input: &'a str) -> Vec<Cmd<'a>> {
        input
            .split(";")
            .map(|cmd| {
                let mut parsed_cmd = cmd.split_whitespace();
                Cmd {
                    comm: parsed_cmd.next(),
                    args: parsed_cmd,
                }
            })
            .collect()
    }

    fn execute(&self) {
        match self.comm {
            Some(comm) => match Command::new(comm).args(self.args.clone()).spawn() {
                Ok(mut child) => {
                    child.wait().expect("Error executing command\n");
                }
                Err(_) => {
                    io::stderr()
                        .write(b"Command not found\n")
                        .expect("Command error\n");
                }
            },
            None => {
                io::stderr()
                    .write(b"Command not found\n")
                    .expect("Command error\n");
            }
        }
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        stdout.write(b"> ")?;
        stdout.flush()?;
        let mut input = String::new();
        stdin.read_line(&mut input)?;

        let command_list = Cmd::parse(input.as_ref());
        for command in command_list {
            command.execute();
        }
    }
}
