use std::env;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

fn main() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed_input = input.trim();
        match trimmed_input.split_once(" ") {
            Some((command, arguments)) => {
                let parsed_args = parse_args(arguments);
                match command {
                    "echo" => println!("{}", parsed_args.join(" ")),
                    "type" => type_fn(&parsed_args.join(" "))?,
                    "cd" => cd_fn(Some(parsed_args))?,
                    _ => run_program(command, Some(parsed_args))?,
                }
            }
            None => match trimmed_input {
                "exit" => break,
                "pwd" => pwd_fn()?,
                "" => {}
                _ => run_program(trimmed_input, None)?,
            },
        }
    }
    Ok(())
}

fn parse_args(arguments: &str) -> Vec<String> {
    let mut parsed_arguments = vec![];
    let mut single_quotes = false;
    let mut double_quotes = false;
    let mut escape = false;
    let mut word = String::new();
    for char in arguments.chars() {
        if char == '\'' {
            if double_quotes || escape {
                word.push(char);
            } else {
                single_quotes = !single_quotes;
            }
        } else if char == '"' {
            if escape || single_quotes {
                word.push(char);
            } else {
                double_quotes = !double_quotes;
            }
        } else if char == '\\' {
            if escape || double_quotes || single_quotes {
                word.push(char);
            } else {
                escape = !escape;
                continue;
            }
        } else if char == ' ' {
            if single_quotes || double_quotes || escape {
                word.push(char);
            } else if !word.is_empty() {
                parsed_arguments.push(word.clone());
                word = String::new();
            }
        } else {
            word.push(char);
        }
        escape = false;
    }
    // push in whatever the last word was
    if !word.is_empty() {
        parsed_arguments.push(word.clone());
    }
    parsed_arguments
}

fn cd_fn(directory: Option<Vec<String>>) -> Result<()> {
    match directory {
        Some(dir) => {
            let dir = dir
                .first()
                .expect("There should be something passed in by this point");
            if dir == "~" {
                let home_dir = env::var("HOME")?;
                env::set_current_dir(home_dir)?;
                return Ok(());
            }

            let path = Path::new(dir);
            if path.exists() {
                env::set_current_dir(path)?;
            } else {
                println!("cd: {}: No such file or directory", dir);
            }
        }
        None => println!("No file or directory passed into cd"),
    }
    Ok(())
}

fn pwd_fn() -> Result<()> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display());
    Ok(())
}

fn type_fn(command: &str) -> Result<()> {
    match command {
        "echo" | "type" | "exit" | "pwd" | "cd" => println!("{} is a shell builtin", command),
        _ => {
            let _ = path_search(command, true)?;
        }
    }
    Ok(())
}

fn path_search(command: &str, verbose: bool) -> Result<Option<PathBuf>> {
    let path = env::var("PATH").unwrap();
    let dirs = path.split(":");
    for dir in dirs {
        let path_str = format!("{dir}/{command}");
        let path = Path::new(&path_str);
        if path.exists() {
            let permissions = path.metadata()?.permissions();
            let is_executable = permissions.mode() & 0o111 != 0;
            if is_executable {
                if verbose {
                    println!("{} is {}", command, path.display());
                }
                return Ok(Some(path.to_path_buf()));
            }
        }
    }
    if verbose {
        println!("{}: not found", command);
    }
    Ok(None)
}

fn run_program(command: &str, arguments: Option<Vec<String>>) -> Result<()> {
    let exc_path = path_search(command, false)?;
    match exc_path {
        Some(_) => {
            if let Some(arguments) = arguments {
                let mut handle = Command::new(command).args(arguments).spawn()?;
                handle.wait()?;
            } else {
                let mut handle = Command::new(command).spawn()?;
                handle.wait()?;
            }
        }
        None => println!("{}: command not found", command),
    }
    Ok(())
}
