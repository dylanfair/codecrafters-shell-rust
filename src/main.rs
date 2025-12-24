use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::ops::Deref;

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{Event, KeyCode, KeyModifiers, read};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor::MoveToColumn, execute};

use builtins::cd::cd_fn;
use builtins::pwd::pwd_fn;
use builtins::type_fn::type_fn;
use input::autocomplete::autocomplete;
use input::utils::parse_input;
use subprocesses::utils::run_program;

mod builtins;
mod input;
mod subprocesses;

const PIPES: [&str; 6] = ["1>", ">", "2>", "1>>", ">>", "2>>"];

pub enum Redirect {
    Stdout,
    Stderr,
    None,
}

fn main() -> Result<()> {
    let mut input = String::new();
    'outer: loop {
        execute!(io::stdout(), MoveToColumn(0))?;
        print!("$ ");
        io::stdout().flush().expect("Could not flush $");

        enable_raw_mode()?;

        loop {
            if let Ok(Event::Key(key_event)) = read() {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Backspace, _) => {
                        if !input.is_empty() {
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                            execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
                        }
                        input.pop();
                    }
                    (KeyCode::Tab, _) => {
                        autocomplete(&mut input)?;
                    }
                    (KeyCode::Enter, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                        disable_raw_mode()?;
                        println!();
                        let mut parsed_input = parse_input(input.trim());
                        input = String::new();
                        if parsed_input.is_empty() {
                            continue 'outer;
                        }

                        if parsed_input.len() == 1 {
                            if let Some(parsed_command) = parsed_input.first() {
                                match parsed_command.deref() {
                                    "exit" => break 'outer,
                                    "pwd" => pwd_fn()?,
                                    "" => {}
                                    _ => run_program(
                                        parsed_command,
                                        None,
                                        &mut None,
                                        &Redirect::None,
                                    )?,
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
                                        continue 'outer;
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
                                        fileoptions.write(true).create(true).append(true);
                                    }
                                    "2>" => {
                                        redirect = Redirect::Stderr;
                                        fileoptions.write(true).create(true);
                                    }
                                    "2>>" => {
                                        redirect = Redirect::Stderr;
                                        fileoptions.write(true).create(true).append(true);
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
                                    "type" => {
                                        type_fn(&args.join(" "), Some(&mut buffer), redirect)?
                                    }
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
                                    "type" => {
                                        type_fn(&parsed_input.join(" "), None, Redirect::None)?
                                    }
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
                            continue 'outer;
                        }
                        continue 'outer;
                    }
                    (KeyCode::Char(c), _) => {
                        input.push(c);
                        print!("{c}");
                        io::stdout().flush().expect("Could not character");
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
