use std::io;
use std::io::Write;

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "cd", "pwd"];

pub fn autocomplete(current_input: &mut String) {
    for builtin in BUILTINS {
        if builtin.starts_with(current_input.as_str()) {
            let to_push = builtin.replace(current_input.as_str(), "");
            for char in to_push.chars() {
                current_input.push(char);
                print!("{char}");
            }
            current_input.push(' ');
            print!(" ");
        }
    }
    io::stdout().flush().expect("Could not flush autocomplete");
}
