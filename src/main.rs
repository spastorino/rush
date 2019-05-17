mod cmd;

use std::convert::TryFrom;
use std::io::{self, Write};

use self::cmd::{Expression, Error};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        stdout.write(b"> ")?;
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        match Expression::try_from(input.as_ref()) {
            Ok(expr) => {
                expr.run().unwrap();
            }

            Err(Error::EmptyLine) => {}

            _ => {}
        }
    }
}
