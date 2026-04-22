use std::io::{self, Write};

mod parser;

use parser::ShellCommand;

fn main() -> Result<(), anyhow::Error> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        ShellCommand::new(input).run()?;
    }
}
