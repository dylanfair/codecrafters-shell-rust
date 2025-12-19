use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Result;

const PIPES: [&str; 6] = ["1>", ">", "2>", "1>>", ">>", "2>>"];

enum Redirect {
    Stdout,
    Stderr,
    None,
}

fn main() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let mut parsed_input = parse_input(input.trim());
        if parsed_input.is_empty() {
            continue;
        }

        if parsed_input.len() == 1 {
            if let Some(parsed_command) = parsed_input.first() {
                match parsed_command.deref() {
                    "exit" => break,
                    "pwd" => pwd_fn()?,
                    "" => {}
                    _ => run_program(parsed_command, None, &mut None, &Redirect::None)?,
                }
            }
        } else {
            let parsed_command = parsed_input.remove(0);

            if let Some((i, pipe)) = parsed_input
                .iter()
                .enumerate()
                .find(|(_, v)| PIPES.contains(&v.as_str()))
            {
                let pipe_target = match parsed_input.get(i + 1) {
                    Some(target) => target,
                    None => {
                        println!("No pipe target found");
                        continue;
                    }
                };

                let mut redirect = Redirect::None;
                let mut fileoptions = OpenOptions::new();
                match pipe.as_str() {
                    ">" | "1>" => {
                        redirect = Redirect::Stdout;
                        fileoptions.write(true).create(true);
                    }
                    ">>" | "1>>" => {
                        redirect = Redirect::Stdout;
                        fileoptions.write(true).append(true);
                    }
                    "2>" => {
                        redirect = Redirect::Stderr;
                        fileoptions.write(true).create(true);
                    }
                    "2>>" => {
                        redirect = Redirect::Stderr;
                        fileoptions.write(true).append(true);
                    }
                    _ => {}
                };

                let file = fileoptions.open(pipe_target)?;
                let mut buffer = BufWriter::new(file);

                let args = parsed_input.drain(0..i).collect::<Vec<String>>();
                match parsed_command.deref() {
                    "echo" => {
                        let mut echo = args.join(" ");
                        match redirect {
                            Redirect::Stdout => {
                                echo.push('\n');
                                buffer.write_all(echo.as_bytes())?;
                            }
                            _ => println!("{echo}"),
                        }
                    }
                    "type" => type_fn(&args.join(" "), Some(&mut buffer), redirect)?,
                    "cd" => cd_fn(Some(args), Some(&mut buffer), redirect)?,
                    _ => run_program(
                        &parsed_command,
                        Some(args),
                        &mut Some(&mut buffer),
                        &redirect,
                    )?,
                }

                buffer.flush()?;
            } else {
                match parsed_command.deref() {
                    "echo" => println!("{}", parsed_input.join(" ")),
                    "type" => type_fn(&parsed_input.join(" "), None, Redirect::None)?,
                    "cd" => cd_fn(Some(parsed_input), None, Redirect::None)?,
                    _ => {
                        run_program(
                            &parsed_command,
                            Some(parsed_input),
                            &mut None,
                            &Redirect::None,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_input(arguments: &str) -> Vec<String> {
    let mut parsed_arguments = vec![];
    let mut single_quotes = false;
    let mut double_quotes = false;
    let mut escape = false;
    let mut word = String::new();
    for char in arguments.chars() {
        match char {
            '\'' => {
                if double_quotes || escape {
                    word.push(char);
                } else {
                    single_quotes = !single_quotes;
                }
            }
            '"' => {
                if escape && double_quotes {
                    word.pop();
                    word.push(char);
                } else if escape || single_quotes {
                    word.push(char);
                } else {
                    double_quotes = !double_quotes;
                }
            }
            '\\' => {
                if escape && double_quotes {
                    word.pop();
                    word.push(char);
                } else if single_quotes {
                    word.push(char);
                } else if double_quotes {
                    word.push(char);
                    escape = !escape;
                    continue;
                } else {
                    escape = !escape;
                    continue;
                }
            }
            ' ' => {
                if single_quotes || double_quotes || escape {
                    word.push(char);
                } else if !word.is_empty() {
                    parsed_arguments.push(word.clone());
                    word = String::new();
                }
            }
            _ => {
                if double_quotes && escape {
                    match char {
                        '$' | '`' | '\n' => {
                            word.pop();
                            word.push(char);
                        }
                        _ => word.push(char),
                    }
                } else {
                    word.push(char);
                }
            }
        }
        escape = false;
    }
    // push in whatever the last word was
    if !word.is_empty() {
        parsed_arguments.push(word.clone());
    }
    parsed_arguments
}

fn cd_fn(
    directory: Option<Vec<String>>,
    buf: Option<&mut BufWriter<File>>,
    redirect: Redirect,
) -> Result<()> {
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
                let no_file_fail = format!("cd: {}: No such file or directory\n", dir);
                match redirect {
                    Redirect::Stderr => {
                        let buffer = buf.expect("If redirecting we should have a file buffer");
                        buffer.write_all(no_file_fail.as_bytes())?;
                    }
                    _ => print!("{no_file_fail}"),
                }
            }
        }
        None => {
            let no_file_passed_in = "No file or directory passed into cd\n";
            match redirect {
                Redirect::Stderr => {
                    let buffer = buf.expect("If redirecting we should have a file buffer");
                    buffer.write_all(no_file_passed_in.as_bytes())?;
                }
                _ => print!("{no_file_passed_in}"),
            }
        }
    }
    Ok(())
}

fn pwd_fn() -> Result<()> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display());
    Ok(())
}

fn type_fn(command: &str, buf: Option<&mut BufWriter<File>>, redirect: Redirect) -> Result<()> {
    match command {
        "echo" | "type" | "exit" | "pwd" | "cd" => {
            let shell_builtin = format!("{} is a shell builtin\n", command);
            match redirect {
                Redirect::Stdout => {
                    let buffer = buf.expect("If redirecting we should have a file buffer");
                    buffer.write_all(shell_builtin.as_bytes())?;
                }
                _ => print!("{shell_builtin}"),
            }
        }
        _ => {
            let _ = path_search(command, true, buf, &redirect)?;
        }
    }
    Ok(())
}

fn path_search(
    command: &str,
    verbose: bool,
    buf: Option<&mut BufWriter<File>>,
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
                        Redirect::Stdout => {
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
        let not_found = format!("{}: Not found\n", command);
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

fn run_program(
    command: &str,
    arguments: Option<Vec<String>>,
    buf: &mut Option<&mut BufWriter<File>>,
    redirect: &Redirect,
) -> Result<()> {
    let exc_path = path_search(command, false, buf.as_deref_mut(), redirect)?;
    match exc_path {
        Some(_) => {
            let mut cmd = Command::new(command);
            match redirect {
                Redirect::Stdout => cmd.stdout(Stdio::piped()),
                Redirect::Stderr => cmd.stderr(Stdio::piped()),
                Redirect::None => cmd.stdout(Stdio::inherit()),
            };

            let mut handle = if let Some(arguments) = arguments {
                cmd.args(arguments).spawn()?
            } else {
                cmd.spawn()?
            };

            let mut output = Vec::new();
            match redirect {
                Redirect::Stdout => {
                    let buffer = buf
                        .as_deref_mut()
                        .expect("If redirecting we should have a file buffer");
                    let mut stdout = handle.stdout.take().expect("Should have an output");
                    stdout.read_to_end(&mut output)?;
                    buffer.write_all(&output)?;

                    return Ok(());
                }
                Redirect::Stderr => {
                    let buffer = buf
                        .as_deref_mut()
                        .expect("If redirecting we should have a file buffer");
                    let mut stderr = handle.stderr.take().expect("Should have an err");
                    stderr.read_to_end(&mut output)?;
                    buffer.write_all(&output)?;

                    return Ok(());
                }
                Redirect::None => {
                    handle.wait()?;
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
    Ok(())
}
