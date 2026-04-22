use std::{
    env,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    process,
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
        _ => println!("{command}: command not found"),
    };

    Ok(())
}

fn find_executable_by_name(name: &str) -> Option<DirEntry> {
    let path = env::var_os("PATH").unwrap().into_string().unwrap();
    let path_dirs = path.split(":").collect::<Vec<_>>();

    for path_dir in path_dirs {
        let dir = fs::read_dir(path_dir);

        if let Ok(read_directory) = dir {
            for directory_entry in read_directory {
                if let Ok(entry) = directory_entry {
                    let file_name = entry.file_name().into_string().unwrap();
                    if name == file_name {
                        if let Ok(metadata) = fs::metadata(entry.path())
                            && metadata.permissions().mode() & 0o111 != 0
                        {
                            return Some(entry);
                        }
                    }
                }
            }
        };
    }

    None
}
