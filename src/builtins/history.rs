use crate::input::utils::Redirect;
use std::io::Write;

use anyhow::Result;

pub struct History {
    list: Vec<String>,
    position: usize,
}

impl History {
    pub fn new() -> History {
        History {
            list: vec![],
            position: 0,
        }
    }

    pub fn add_entry(&mut self, entry: String) {
        self.list.push(entry);
        self.position = self.list.len();
    }

    pub fn move_up(&mut self) -> Option<&String> {
        if self.position == 0 {
            return None;
        }
        self.position = self.position.saturating_sub(1);
        self.list.get(self.position)
    }

    pub fn move_down(&mut self) -> Option<&String> {
        self.position = self.position.saturating_add(1);
        if self.position > self.list.len() {
            self.position = self.list.len();
            return None;
        }
        self.list.get(self.position)
    }
}

pub fn history_fn(
    history: &mut History,
    arguments: Vec<String>,
    buf: Option<&mut Vec<u8>>,
    redirect: &Redirect,
) -> Result<()> {
    let mut history_display = String::new();

    if arguments.is_empty() {
        for (i, command) in history.list.iter().enumerate() {
            history_display.push_str(&format!("  {}  {command}\n", i + 1));
        }
    } else {
        let history_n = arguments.first().unwrap();

        match history_n.parse::<usize>() {
            Ok(history_n) => {
                if history_n > history.list.len() {
                    let history_n_too_large = format!(
                        "Number provided is larger than current history: {}\n",
                        history.list.len()
                    );
                    match redirect {
                        Redirect::Stderr => {
                            let buffer = buf.expect("If redirecting we should have a file buffer");
                            buffer.write_all(history_n_too_large.as_bytes())?;
                        }
                        _ => print!("{history_n_too_large}"),
                    }
                    return Ok(());
                }

                for i in history.list.len() - history_n..history.list.len() {
                    let command = history
                        .list
                        .get(i)
                        .expect("Should be here since we checked length");
                    history_display.push_str(&format!(" {}  {command}\n", i + 1));
                }
            }
            Err(_) => {
                let history_n_parse_fail = "History needs to be provided a number\n";
                match redirect {
                    Redirect::Stderr => {
                        let buffer = buf.expect("If redirecting we should have a file buffer");
                        buffer.write_all(history_n_parse_fail.as_bytes())?;
                    }
                    _ => print!("{history_n_parse_fail}"),
                }
                return Ok(());
            }
        }
    }

    match redirect {
        Redirect::Stdout | Redirect::Pipe => {
            let buffer = buf.expect("If redirecting we should have a file buffer");
            buffer.write_all(history_display.as_bytes())?;
        }
        _ => print!("{history_display}"),
    }
    Ok(())
}
