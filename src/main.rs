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
            Some((command, arguments)) => match command {
                "echo" => println!("{}", arguments),
                "type" => type_fn(arguments)?,
                _ => run_program(command, Some(arguments))?,
            },
            None => match trimmed_input {
                "exit" => break,
                _ => run_program(trimmed_input, None)?,
            },
        }
    }
    Ok(())
}

fn type_fn(command: &str) -> Result<()> {
    match command {
        "echo" | "type" | "exit" => println!("{} is a shell builtin", command),
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

fn run_program(command: &str, arguments: Option<&str>) -> Result<()> {
    let exc_path = path_search(command, false)?;
    match exc_path {
        Some(exc_path) => {
            if let Some(arguments) = arguments {
                let mut handle = Command::new(exc_path).args(arguments.split(" ")).spawn()?;
                handle.wait()?;
            } else {
                let mut handle = Command::new(exc_path).spawn()?;
                handle.wait()?;
            }
        }
        None => println!("{}: command not found", command),
    }
    Ok(())
}
