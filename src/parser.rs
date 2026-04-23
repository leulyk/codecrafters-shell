use anyhow::{Result, anyhow};
use std::{
    env,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{self, Command},
};

const BUILTINS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

pub struct ShellCommand<'a> {
    command: &'a str,
    args: Vec<String>,
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
    Pwd,
    Cd,
}

impl<'a> ShellCommand<'a> {
    pub fn new(input: &'a str) -> Self {
        let command_delim_index = input.find(" ");
        let command = &input[..command_delim_index.unwrap_or(input.len())];
        let args = match command_delim_index {
            Some(index) => Self::tokenize_args(&input[index + 1..]),
            None => vec![],
        };

        let command_type = if BUILTINS.contains(&command) {
            match command {
                "cd" => Some(CommandType::Builtin(Builtins::Cd)),
                "pwd" => Some(CommandType::Builtin(Builtins::Pwd)),
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

    pub fn run(&mut self) -> Result<(), anyhow::Error> {
        match &self.command_type {
            Some(CommandType::Builtin(b)) => match b {
                Builtins::Exit => process::exit(0),
                Builtins::Echo => println!("{}", self.args.join(" ")),
                Builtins::Type => self.parse_type(),
                Builtins::Pwd => println!("{}", env::current_dir()?.display()),
                Builtins::Cd => self.handle_directory_change()?,
            },
            Some(CommandType::Executable) => {
                Command::new(self.command).args(&self.args).status()?;
            }
            None => println!("{}: command not found", self.command),
        };

        Ok(())
    }

    fn parse_type(&self) {
        let args_str = self.args.join(" ");

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

    fn handle_directory_change(&self) -> Result<()> {
        match self.args.len() {
            0 => env::set_current_dir(fetch_home_path()?)?,
            1 => {
                let current_directory = env::current_dir()?;
                let arg = match self.args[0].as_str() {
                    "~" => fetch_home_path()?.display().to_string(),
                    _ if self.args[0].starts_with("./") => {
                        current_directory.display().to_string() + "/" + &self.args[0][2..]
                    }
                    _ if self.args[0].starts_with("../") => {
                        let mut parent_index = 3;
                        let mut parent = current_directory.parent().unwrap();
                        while self.args[0][parent_index..].starts_with("../") {
                            parent = parent.parent().unwrap();
                            parent_index += 3;
                        }
                        parent.display().to_string() + "/" + &self.args[0][parent_index..]
                    }
                    _ if self.args[0].starts_with("/") => self.args[0].to_string(),
                    _ => current_directory.display().to_string() + "/" + &self.args[0],
                };

                env::set_current_dir(arg).unwrap_or_else(|_| {
                    println!("cd: {}: No such file or directory", self.args[0])
                });
            }
            _ => println!("Too many args for cd command"),
        }

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

    fn tokenize_args(args_str: &str) -> Vec<String> {
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;
        let mut escaping = false;
        let mut buffer: Vec<char> = vec![];
        let mut args: Vec<String> = vec![];

        for ch in args_str.chars() {
            if escaping {
                buffer.push(ch);
                escaping = false;
                continue;
            }

            match ch {
                '\\' => escaping = true,
                '\'' => {
                    if in_double_quotes {
                        buffer.push(ch);
                    } else {
                        in_single_quotes = !in_single_quotes;
                    }
                }
                '"' => {
                    if in_single_quotes {
                        buffer.push(ch);
                    } else {
                        in_double_quotes = !in_double_quotes
                    }
                }
                ' ' => {
                    if in_single_quotes || in_double_quotes {
                        buffer.push(ch);
                    } else if !buffer.is_empty() {
                        args.push(buffer.iter().collect());
                        buffer.clear();
                    }
                }
                _ => {
                    buffer.push(ch);
                }
            }
        }

        if !buffer.is_empty() {
            args.push(buffer.iter().collect());
        }

        args
    }
}

fn fetch_home_path() -> Result<PathBuf> {
    env::home_dir().ok_or_else(|| anyhow!("Failed to open home directory..."))
}
