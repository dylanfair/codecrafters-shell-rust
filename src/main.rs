#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        match command.trim().split_once(" ") {
            Some((command, arguments)) => match command {
                "echo" => println!("{}", arguments),
                _ => println!("{}: command not found", command),
            },
            None => match command.trim() {
                "exit" => break,
                _ => println!("{}: command not found", command),
            },
        }
    }
}
