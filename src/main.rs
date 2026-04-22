use std::io::{self, Write};

mod parser;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read from stdin...");

        let command = command.trim();

        let _ = parser::parse_command(&command);
    }
}
