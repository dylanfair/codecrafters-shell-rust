use std::env;
use std::io::Write;

use anyhow::Result;

use crate::input::utils::Redirect;

pub fn pwd_fn(buf: Option<&mut Vec<u8>>, redirect: &Redirect) -> Result<()> {
    let current_dir = env::current_dir()?;
    let pwd_display = format!("{} \n", current_dir.display());
    match redirect {
        Redirect::Stdout | Redirect::Pipe => {
            let buffer = buf.expect("If redirecting we should have a file buffer");
            buffer.write_all(pwd_display.as_bytes())?;
        }
        _ => print!("{pwd_display}"),
    }
    Ok(())
}
