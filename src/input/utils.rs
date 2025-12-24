use std::fs::OpenOptions;
use std::io::{self, Write};

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode};

use crate::builtins::cd::cd_fn;
use crate::builtins::pwd::pwd_fn;
use crate::builtins::type_fn::type_fn;
use crate::input::autocomplete::autocomplete;
use crate::input::inputblock::InputBlock;
use crate::subprocesses::utils::run_program;

#[derive(Clone, PartialEq, Debug)]
pub enum Redirect {
    Stdout,
    Stderr,
    Pipe,
    None,
}

#[derive(Clone, Debug)]
pub enum RedirectType {
    Create,
    Append,
    None,
}

#[derive(Clone, Debug)]
pub struct RedirectOptions {
    redirect: Redirect,
    redirect_type: RedirectType,
    redirect_location: String,
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
            let parsed_input = parse_input(input.trim());

            *input = String::new();
            if parsed_input.is_empty() {
                return Ok(InputLoop::ContinueOuter);
            }

            let mut previous_output = None;
            for input_block in parsed_input {
                let args = input_block.args;
                let mut buffer = vec![];

                let redirect = input_block.redirect_options.redirect;
                let redirect_bool = redirect == Redirect::Stdout || redirect == Redirect::Stderr;
                let redirect_location = input_block.redirect_options.redirect_location;
                let mut fileoptions = OpenOptions::new();

                if redirect_bool && redirect_location.is_empty() {
                    println!("No redirect target found");
                    return Ok(InputLoop::ContinueOuter);
                }

                if redirect_bool {
                    match input_block.redirect_options.redirect_type {
                        RedirectType::Create => {
                            fileoptions.write(true).create(true);
                        }
                        RedirectType::Append => {
                            fileoptions.write(true).create(true).append(true);
                        }
                        RedirectType::None => {
                            println!("No redirect type found");
                            return Ok(InputLoop::ContinueOuter);
                        }
                    }
                }

                match input_block.command.as_str() {
                    "echo" => {
                        let mut echo = args.join(" ");
                        match redirect {
                            Redirect::Stdout | Redirect::Pipe => {
                                echo.push('\n');
                                buffer.write_all(echo.as_bytes())?;
                            }
                            _ => println!("{echo}"),
                        }
                    }
                    "exit" => return Ok(InputLoop::Exit),
                    "pwd" => pwd_fn(Some(&mut buffer), &redirect)?,
                    "type" => type_fn(&args.join(" "), Some(&mut buffer), &redirect)?,
                    "cd" => cd_fn(Some(args), Some(&mut buffer), &redirect)?,
                    "" => {}
                    _ => {
                        let child_stdout = run_program(
                            &input_block.command,
                            Some(args),
                            previous_output,
                            &mut Some(&mut buffer),
                            &redirect,
                        )?;
                        previous_output = child_stdout;
                    }
                }

                match redirect {
                    Redirect::Stdout | Redirect::Stderr => {
                        let mut file = fileoptions.open(redirect_location)?;
                        file.write_all(&buffer)?;
                    }
                    _ => {}
                }

                if input_block.piped {
                    continue;
                } else {
                    return Ok(InputLoop::ContinueOuter);
                }
            }
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

pub fn parse_input(arguments: &str) -> Vec<InputBlock> {
    let mut input_blocks = vec![];
    let mut parsed_command = String::new();
    let mut parsed_arguments = vec![];
    let mut redirect = Redirect::None;
    let mut redirect_type = RedirectType::None;
    let mut redirect_location = String::new();

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
                    if parsed_command.is_empty() {
                        parsed_command = word;
                    } else if redirect != Redirect::None {
                        redirect_location = word;
                    } else {
                        match word.as_str() {
                            "|" => {
                                let input_block = InputBlock::new(
                                    parsed_command.clone(),
                                    parsed_arguments.clone(),
                                    RedirectOptions {
                                        redirect: Redirect::Pipe,
                                        redirect_type,
                                        redirect_location: redirect_location.clone(),
                                    },
                                    true,
                                );
                                input_blocks.push(input_block);

                                // Reset all for a new input block
                                parsed_command = String::new();
                                parsed_arguments = vec![];
                                redirect = Redirect::None;
                                redirect_type = RedirectType::None;
                                redirect_location = String::new();
                            }
                            ">" | "1>" => {
                                redirect = Redirect::Stdout;
                                redirect_type = RedirectType::Create;
                            }
                            ">>" | "1>>" => {
                                redirect = Redirect::Stdout;
                                redirect_type = RedirectType::Append;
                            }
                            "2>" => {
                                redirect = Redirect::Stderr;
                                redirect_type = RedirectType::Create;
                            }
                            "2>>" => {
                                redirect = Redirect::Stderr;
                                redirect_type = RedirectType::Append;
                            }
                            _ => {
                                parsed_arguments.push(word.clone());
                            }
                        }
                    }
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
        if parsed_command.is_empty() {
            parsed_command = word;
        } else if redirect != Redirect::None {
            redirect_location = word;
        } else {
            parsed_arguments.push(word.clone());
        }

        let input_block = InputBlock::new(
            parsed_command,
            parsed_arguments,
            RedirectOptions {
                redirect,
                redirect_type,
                redirect_location,
            },
            false,
        );
        input_blocks.push(input_block);
    }
    input_blocks
}
