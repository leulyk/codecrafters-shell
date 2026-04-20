#[allow(unused_imports)]
use std::{
    io::{self, Write},
    process,
};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read from stdin...");

        let command = command.trim();
        parse_command(&command);
    }
}

fn parse_command(command: &str) {
    if command == "exit" {
        process::exit(0);
    }
    println!("{command}: command not found");
}
