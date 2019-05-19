mod cmd;

use std::convert::TryFrom;
use std::io::{self, Write};

use self::cmd::{Error, Expression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        stdout.write(b"> ")?;
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        match Expression::try_from(input.as_ref()) {
            Ok(expr) => {
                let _ = expr.run();
            }
            Err(Error::EmptyLine) => {}
            _ => {}
        }
    }
}
