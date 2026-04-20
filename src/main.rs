#[allow(unused_imports)]
use std::{
    io::{self, Write},
    process,
};

const BUILTINS: [&str; 3] = ["exit", "echo", "type"];

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
    let split_index = command.find(" ");
    let mut arguments = String::new();
    let command = match split_index {
        Some(index) => {
            arguments = command[index + 1..].to_string();
            &command[..index]
        }
        None => command,
    };

    match command {
        "exit" => process::exit(0),
        "echo" => println!("{arguments}"),
        "type" => {
            if BUILTINS.contains(&arguments.as_str()) {
                println!("{arguments} is a shell builtin");
            } else if arguments != "" {
                println!("{arguments}: not found");
            }
        }
        _ => println!("{command}: command not found"),
    }
}
