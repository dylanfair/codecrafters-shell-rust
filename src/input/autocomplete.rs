use std::env;
use std::fs::read_dir;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::builtins::history::History;
use crate::input::utils::{InputLoop, handle_key_press};

const BUILTINS: [&str; 6] = ["echo", "exit", "type", "cd", "pwd", "history"];

fn push_completed(completed: &str, current_input: &mut String) {
    let to_push = completed.replace(current_input.as_str(), "");
    for char in to_push.chars() {
        current_input.push(char);
        print!("{char}");
    }
    current_input.push(' ');
    print!(" ");
    io::stdout().flush().expect("Could not flush autocomplete");
}

pub fn autocomplete(current_input: &mut String, history: &mut History) -> Result<InputLoop> {
    let mut potential_matches: Vec<String> = vec![];

    // First check builtins
    for builtin in BUILTINS {
        if builtin.starts_with(current_input.as_str())
            && !potential_matches.contains(&builtin.to_string())
        {
            potential_matches.push(builtin.to_string());
        }
    }

    // Then search path
    let path = env::var("PATH").unwrap();
    let dirs = path.split(":");
    for dir in dirs {
        let dir_path = Path::new(dir);
        if dir_path.exists() {
            for entry in (read_dir(dir_path)?).flatten() {
                let entry_file = entry.file_name();
                let entry_str = entry_file.into_string().unwrap();
                if entry_str.starts_with(current_input.as_str()) {
                    let permissions = entry.metadata()?.permissions();
                    let is_executable = permissions.mode() & 0o111 != 0;
                    if is_executable && !potential_matches.contains(&entry_str) {
                        potential_matches.push(entry_str);
                    }
                }
            }
        }
    }

    if potential_matches.len() == 1 {
        push_completed(potential_matches.first().unwrap(), current_input);
    } else if potential_matches.len() > 1 {
        print!("\x07");
        io::stdout().flush().expect("Could not flush bell");

        let longest_common_prefix =
            find_longest_common_prefix(&potential_matches, current_input.len());
        if !longest_common_prefix.is_empty() {
            current_input.push_str(&longest_common_prefix);
            print!("{longest_common_prefix}");
            io::stdout()
                .flush()
                .expect("Could not flush longest_common_prefix");
        }

        if let Ok(Event::Key(key_event)) = read() {
            if key_event.code == KeyCode::Tab {
                potential_matches.sort();
                let potential_commands = potential_matches.join("  ");
                disable_raw_mode()?;
                println!();
                println!("{potential_commands}");
                print!("$ {current_input}");
                io::stdout()
                    .flush()
                    .expect("Could not flush potential commands");
                enable_raw_mode()?;
            } else {
                return handle_key_press(current_input, key_event, history);
            }
        }
    } else {
        print!("\x07");
        io::stdout().flush().expect("Could not flush bell");
    }

    Ok(InputLoop::ContinueInner)
}

fn find_longest_common_prefix(potential_matches: &[String], start_from: usize) -> String {
    let mut longest_common_prefix = String::new();
    let mut current_char_place = start_from;

    'outer: loop {
        let mut current_char = '\0';
        for i in 0..potential_matches.len() {
            let pmatch = potential_matches.get(i).unwrap();
            match pmatch.chars().nth(current_char_place) {
                Some(pmatch_char) => {
                    if current_char == '\0' {
                        current_char = pmatch_char;
                    } else if pmatch_char != current_char {
                        break 'outer;
                    }
                }
                None => break 'outer,
            }
        }
        longest_common_prefix.push(current_char);
        current_char_place += 1;
    }

    longest_common_prefix
}
