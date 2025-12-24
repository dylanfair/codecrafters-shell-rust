use std::env;
use std::fs::read_dir;
use std::io;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;

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
    // First check builtins
    for builtin in BUILTINS {
        if builtin.starts_with(current_input.as_str()) {
            push_completed(builtin, current_input);
            return Ok(());
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
                let entry_str = entry_file
                    .to_str()
                    .expect("File name should have no trouble going to str");
                if entry_str.starts_with(current_input.as_str()) {
                    let permissions = entry.metadata()?.permissions();
                    let is_executable = permissions.mode() & 0o111 != 0;
                    if is_executable {
                        push_completed(entry_str, current_input);
                        return Ok(());
                    }
                }
            }
        }
    }

    print!("\x07");
    io::stdout().flush().expect("Could not flush bell");
    Ok(())
}
