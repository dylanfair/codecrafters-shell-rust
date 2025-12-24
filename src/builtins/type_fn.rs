use crate::input::utils::Redirect;
use std::io::Write;

use crate::subprocesses::utils::path_search;

use anyhow::Result;

pub fn type_fn(command: &str, buf: Option<&mut Vec<u8>>, redirect: &Redirect) -> Result<()> {
    match command {
        "echo" | "type" | "exit" | "pwd" | "cd" => {
            let shell_builtin = format!("{} is a shell builtin\n", command);
            match redirect {
                Redirect::Stdout | Redirect::Pipe => {
                    let buffer = buf.expect("If redirecting we should have a file buffer");
                    buffer.write_all(shell_builtin.as_bytes())?;
                }
                _ => print!("{shell_builtin}"),
            }
        }
        _ => {
            let _ = path_search(command, true, buf, redirect)?;
        }
    }
    Ok(())
}
