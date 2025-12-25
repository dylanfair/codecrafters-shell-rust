use crate::input::utils::Redirect;
use std::io::Write;

use anyhow::Result;

pub fn history_fn(
    history: &mut [String],
    arguments: Vec<String>,
    buf: Option<&mut Vec<u8>>,
    redirect: &Redirect,
) -> Result<()> {
    let mut history_display = String::new();

    if arguments.is_empty() {
        for (i, command) in history.iter().enumerate() {
            history_display.push_str(&format!("  {}  {command}\n", i + 1));
        }
    } else {
        let history_n = arguments.first().unwrap();

        match history_n.parse::<usize>() {
            Ok(history_n) => {
                if history_n > history.len() {
                    let history_n_too_large = format!(
                        "Number provided is larger than current history: {}\n",
                        history.len()
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

                for i in history.len() - history_n..history.len() {
                    let command = history
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
