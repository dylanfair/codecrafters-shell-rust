use crate::input::utils::Redirect;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use anyhow::Result;

pub struct History {
    list: Vec<String>,
    position: usize,
    append_start: usize,
}

impl History {
    pub fn new() -> History {
        History {
            list: vec![],
            position: 0,
            append_start: 0,
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
        let arg = arguments.first().unwrap();

        match arg.as_str() {
            "-r" => match arguments.get(1) {
                Some(file) => {
                    let content = fs::read_to_string(file)?;
                    for line in content.lines() {
                        history.add_entry(line.to_string());
                    }
                }
                None => {
                    let missing_file = "Need to be sent a file\n";
                    match redirect {
                        Redirect::Stderr => {
                            let buffer = buf.expect("If redirecting we should have a file buffer");
                            buffer.write_all(missing_file.as_bytes())?;
                        }
                        _ => print!("{missing_file}"),
                    }
                    return Ok(());
                }
            },
            "-w" => match arguments.get(1) {
                Some(file) => {
                    let mut file_handler = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(file)?;

                    for line in &history.list {
                        writeln!(file_handler, "{line}")?;
                    }
                }
                None => {
                    let missing_file = "Need to be sent a file\n";
                    match redirect {
                        Redirect::Stderr => {
                            let buffer = buf.expect("If redirecting we should have a file buffer");
                            buffer.write_all(missing_file.as_bytes())?;
                        }
                        _ => print!("{missing_file}"),
                    }
                    return Ok(());
                }
            },
            "-a" => match arguments.get(1) {
                Some(file) => {
                    let mut file_handler =
                        OpenOptions::new().create(true).append(true).open(file)?;

                    for i in history.append_start..history.list.len() {
                        let entry = history.list.get(i).expect("getting within lsit len");
                        writeln!(file_handler, "{entry}")?;
                    }
                    history.append_start = history.list.len();
                }
                None => {
                    let missing_file = "Need to be sent a file\n";
                    match redirect {
                        Redirect::Stderr => {
                            let buffer = buf.expect("If redirecting we should have a file buffer");
                            buffer.write_all(missing_file.as_bytes())?;
                        }
                        _ => print!("{missing_file}"),
                    }
                    return Ok(());
                }
            },
            _ => match arg.parse::<usize>() {
                Ok(history_n) => {
                    if history_n > history.list.len() {
                        let history_n_too_large = format!(
                            "Number provided is larger than current history: {}\n",
                            history.list.len()
                        );
                        match redirect {
                            Redirect::Stderr => {
                                let buffer =
                                    buf.expect("If redirecting we should have a file buffer");
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
            },
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
