use std::fs::OpenOptions;
use std::io::{self, Write};
use std::ops::Deref;

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode};

use crate::builtins::cd::cd_fn;
use crate::builtins::pwd::pwd_fn;
use crate::builtins::type_fn::type_fn;
use crate::input::autocomplete::autocomplete;
use crate::subprocesses::utils::run_program;

const PIPES: [&str; 6] = ["1>", ">", "2>", "1>>", ">>", "2>>"];

pub enum Redirect {
    Stdout,
    Stderr,
    None,
}

pub enum InputLoop {
    ContinueOuter,
    ContinueInner,
    Exit,
}

pub fn handle_key_press(input: &mut String, key_event: KeyEvent) -> Result<InputLoop> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Backspace, _) => {
            if !input.is_empty() {
                execute!(io::stdout(), cursor::MoveLeft(1))?;
                execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
            }
            input.pop();
        }
        (KeyCode::Tab, _) => {
            return autocomplete(input);
        }
        (KeyCode::Enter, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
            disable_raw_mode()?;
            println!();
            let mut parsed_input = parse_input(input.trim());
            *input = String::new();
            if parsed_input.is_empty() {
                return Ok(InputLoop::ContinueOuter);
            }

            if parsed_input.len() == 1 {
                if let Some(parsed_command) = parsed_input.first() {
                    match parsed_command.deref() {
                        "exit" => return Ok(InputLoop::Exit),
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
                            return Ok(InputLoop::ContinueOuter);
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

                    let mut file = fileoptions.open(pipe_target)?;
                    let mut buffer = vec![];

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

                    file.write_all(&buffer)?;
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
                return Ok(InputLoop::ContinueOuter);
            }
            return Ok(InputLoop::ContinueOuter);
        }
        (KeyCode::Char(c), _) => {
            input.push(c);
            print!("{c}");
            io::stdout().flush().expect("Could not character");
        }
        _ => {}
    }

    Ok(InputLoop::ContinueInner)
}

pub fn parse_input(arguments: &str) -> Vec<String> {
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
