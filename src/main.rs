use std::io::{self, Write};

use anyhow::Result;
use crossterm::event::{Event, read};
use crossterm::terminal::enable_raw_mode;
use crossterm::{cursor::MoveToColumn, execute};

use crate::builtins::history::History;
use crate::input::utils::{InputLoop, handle_key_press};

mod builtins;
mod input;
mod subprocesses;

fn main() -> Result<()> {
    let mut input = String::new();
    let mut history = match History::read_from_env() {
        Ok(history) => history,
        Err(_) => History::new(),
    };

    'outer: loop {
        execute!(io::stdout(), MoveToColumn(0))?;
        print!("$ ");
        io::stdout().flush().expect("Could not flush $");

        enable_raw_mode()?;

        loop {
            if let Ok(Event::Key(key_event)) = read() {
                let inputloop = handle_key_press(&mut input, key_event, &mut history)?;
                match inputloop {
                    InputLoop::ContinueOuter => continue 'outer,
                    InputLoop::ContinueInner => {}
                    InputLoop::Exit => break 'outer,
                }
            }
        }
    }
    history.write_to_env()?;
    Ok(())
}
