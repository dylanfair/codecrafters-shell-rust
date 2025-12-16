#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed_input = input.trim();
        match trimmed_input.split_once(" ") {
            Some((command, arguments)) => match command {
                "echo" => println!("{}", arguments),
                _ => println!("{}: command not found", command),
            },
            None => match trimmed_input {
                "exit" => break,
                _ => println!("{}: command not found", trimmed_input),
            },
        }
    }
}
