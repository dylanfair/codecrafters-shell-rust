use std::env;
use std::fs::read_dir;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "cd", "pwd"];

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

pub fn autocomplete(current_input: &mut String) -> Result<()> {
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
        if let Ok(Event::Key(key_event)) = read()
            && key_event.code == KeyCode::Tab
        {
            potential_matches.sort();
            let potential_commands = potential_matches.join("  ");
            disable_raw_mode()?;
            println!();
            println!("{potential_commands}");
            print!("$ {current_input}");
            io::stdout().flush().expect("Could not flush bell");
            enable_raw_mode()?;
        }
    } else {
        print!("\x07");
        io::stdout().flush().expect("Could not flush bell");
    }

    Ok(())
}
