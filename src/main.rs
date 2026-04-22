use std::io::{self, Write};

mod parser;

fn main() -> Result<(), anyhow::Error> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command)?;

        let command = command.trim();

        parser::parse_command(&command)?;
    }
}
