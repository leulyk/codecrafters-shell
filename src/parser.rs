use std::{
    env,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    process::{self, Command},
};

const BUILTINS: [&str; 3] = ["exit", "echo", "type"];

pub fn parse_command(command: &str) -> Result<(), anyhow::Error> {
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
            } else {
                if let Some(entry) = find_executable_by_name(&arguments) {
                    println!(
                        "{} is {}",
                        arguments,
                        entry.path().into_os_string().into_string().unwrap()
                    );
                } else if arguments != "" {
                    println!("{arguments}: not found");
                }
            }
        }
        _ => {
            if find_executable_by_name(&command).is_some() {
                let args = arguments.split(" ").collect::<Vec<_>>();
                Command::new(command).args(args).status()?;
            } else {
                println!("{command}: command not found");
            }
        }
    };

    Ok(())
}

fn find_executable_by_name(name: &str) -> Option<DirEntry> {
    let paths = env::var_os("PATH")?;

    for path in env::split_paths(&paths) {
        let Ok(read_directory) = fs::read_dir(path) else {
            continue;
        };

        for directory_entry in read_directory {
            let Ok(entry) = directory_entry else { continue };

            if let Some(file_name) = entry.file_name().to_str()
                && name == file_name
            {
                if let Ok(metadata) = entry.metadata()
                    && metadata.permissions().mode() & 0o111 != 0
                {
                    return Some(entry);
                }
            }
        }
    }

    None
}
