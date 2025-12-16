use std::env;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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
                _ => println!("{}: command not found", command),
            },
            None => match trimmed_input {
                "exit" => break,
                _ => println!("{}: command not found", trimmed_input),
            },
        }
    }
    Ok(())
}

fn type_fn(command: &str) -> Result<()> {
    match command {
        "echo" | "type" | "exit" => println!("{} is a shell builtin", command),
        _ => path_search(command)?,
    }
    Ok(())
}

fn path_search(command: &str) -> Result<()> {
    let path = env::var("PATH").unwrap();
    let dirs = path.split(":");
    for dir in dirs {
        let path_str = format!("{dir}/{command}");
        let path = Path::new(&path_str);
        if path.exists() {
            let permissions = path.metadata()?.permissions();
            let is_executable = permissions.mode() & 0o111 != 0;
            if is_executable {
                println!("{} is {}", command, path.display());
                return Ok(());
            }
        }
    }
    println!("{}: not found", command);
    Ok(())
}
