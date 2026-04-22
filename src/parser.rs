use std::{
    env,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    process::{self, Command},
};

const BUILTINS: [&str; 3] = ["exit", "echo", "type"];

pub struct ShellCommand<'a> {
    command: &'a str,
    args: Vec<&'a str>,
    command_type: Option<CommandType>,
}

pub enum CommandType {
    Builtin(Builtins),
    Executable,
}

pub enum Builtins {
    Exit,
    Echo,
    Type,
}

impl<'a> ShellCommand<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut all_args = input.split(" ");
        let command = all_args.next().unwrap_or("");
        let args = all_args.collect::<Vec<_>>();

        let command_type = if BUILTINS.contains(&command) {
            match command {
                "exit" => Some(CommandType::Builtin(Builtins::Exit)),
                "echo" => Some(CommandType::Builtin(Builtins::Echo)),
                "type" => Some(CommandType::Builtin(Builtins::Type)),
                _ => None,
            }
        } else if Self::is_executable(command).is_some() {
            Some(CommandType::Executable)
        } else {
            None
        };

        Self {
            command,
            args,
            command_type,
        }
    }

    pub fn parse(&self) -> Result<(), anyhow::Error> {
        let args_str = self.args.join(" ");
        match &self.command_type {
            Some(CommandType::Builtin(b)) => match b {
                Builtins::Exit => process::exit(0),
                Builtins::Echo => println!("{}", args_str),
                Builtins::Type => {
                    if BUILTINS.contains(&args_str.as_str()) {
                        println!("{args_str} is a shell builtin");
                    } else {
                        if let Some(entry) = Self::is_executable(&args_str) {
                            println!(
                                "{} is {}",
                                args_str,
                                entry.path().into_os_string().into_string().unwrap()
                            );
                        } else if args_str != "" {
                            println!("{args_str}: not found");
                        }
                    }
                }
            },
            Some(CommandType::Executable) => {
                Command::new(self.command).args(&self.args).status()?;
            }
            None => println!("{}: command not found", self.command),
        };

        Ok(())
    }

    fn is_executable(name: &str) -> Option<DirEntry> {
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
}
