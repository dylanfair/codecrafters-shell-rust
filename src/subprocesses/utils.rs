use std::env;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{ChildStdout, Command, Stdio};

use anyhow::Result;

use crate::input::utils::Redirect;

pub enum OutputHandle {
    ChildPipe(ChildStdout),
    ChildBuffer(Vec<u8>),
}

pub fn path_search(
    command: &str,
    verbose: bool,
    buf: Option<&mut Vec<u8>>,
    redirect: &Redirect,
) -> Result<Option<PathBuf>> {
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
                    let exe_is_path = format!("{} is {}\n", command, path.display());
                    match redirect {
                        Redirect::Stdout | Redirect::Pipe => {
                            let buffer = buf.expect("If redirecting we should have a file buffer");
                            buffer.write_all(exe_is_path.as_bytes())?;
                        }
                        _ => print!("{exe_is_path}"),
                    }
                }
                return Ok(Some(path.to_path_buf()));
            }
        }
    }
    if verbose {
        let not_found = format!("{}: not found\n", command);
        match redirect {
            Redirect::Stderr => {
                let buffer = buf.expect("If redirecting we should have a file buffer");
                buffer.write_all(not_found.as_bytes())?
            }
            _ => print!("{not_found}"),
        }
    }
    Ok(None)
}

pub fn run_program(
    command: &str,
    arguments: Option<Vec<String>>,
    piped_input: Option<OutputHandle>,
    buf: &mut Option<&mut Vec<u8>>,
    redirect: &Redirect,
) -> Result<Option<OutputHandle>> {
    let exc_path = path_search(command, false, buf.as_deref_mut(), redirect)?;
    match exc_path {
        Some(_) => {
            let mut cmd = Command::new(command);
            match redirect {
                Redirect::Stdout | Redirect::Pipe => cmd.stdout(Stdio::piped()),
                Redirect::Stderr => cmd.stderr(Stdio::piped()),
                Redirect::None => cmd.stdout(Stdio::inherit()),
            };

            let childbuffer = if let Some(outputhandle) = piped_input {
                match outputhandle {
                    OutputHandle::ChildPipe(childstdout) => {
                        let childst = childstdout;
                        cmd.stdin(childst);
                        vec![]
                    }
                    OutputHandle::ChildBuffer(childbuffer) => {
                        cmd.stdin(Stdio::piped());
                        childbuffer
                    }
                }
            } else {
                vec![]
            };

            let mut handle = if let Some(arguments) = arguments {
                cmd.args(arguments).spawn()?
            } else {
                cmd.spawn()?
            };

            if !childbuffer.is_empty() {
                let mut stdin = handle.stdin.take().expect("Failed to open stdin");
                stdin.write_all(&childbuffer)?;
                drop(stdin);
            }

            let mut output = Vec::new();
            match redirect {
                Redirect::Pipe => {
                    let stdout = handle.stdout.expect("Should have an output");
                    return Ok(Some(OutputHandle::ChildPipe(stdout)));
                }
                Redirect::Stdout => {
                    let buffer = buf
                        .as_deref_mut()
                        .expect("If redirecting we should have a file buffer");
                    let mut stdout = handle.stdout.take().expect("Should have an output");
                    stdout.read_to_end(&mut output)?;
                    buffer.write_all(&output)?;

                    return Ok(None);
                }
                Redirect::Stderr => {
                    let buffer = buf
                        .as_deref_mut()
                        .expect("If redirecting we should have a file buffer");
                    let mut stderr = handle.stderr.take().expect("Should have an err");
                    stderr.read_to_end(&mut output)?;
                    buffer.write_all(&output)?;

                    return Ok(None);
                }
                Redirect::None => {
                    handle.wait()?;
                    return Ok(None);
                }
            }
        }
        None => {
            let command_not_found = format!("{}: command not found\n", command);
            match redirect {
                Redirect::Stderr => {
                    let buffer = buf
                        .as_deref_mut()
                        .expect("If redirecting we should have a file buffer");
                    buffer.write_all(command_not_found.as_bytes())?;
                }
                _ => print!("{command_not_found}"),
            }
        }
    }
    Ok(None)
}
