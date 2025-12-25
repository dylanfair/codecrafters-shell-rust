use std::env;
use std::io::Write;
use std::path::Path;

use anyhow::Result;

use crate::input::utils::Redirect;

pub fn cd_fn(directory: Vec<String>, buf: Option<&mut Vec<u8>>, redirect: &Redirect) -> Result<()> {
    if !directory.is_empty() {
        let dir = directory
            .first()
            .expect("There should be something passed in by this point");
        if dir == "~" {
            let home_dir = env::var("HOME")?;
            env::set_current_dir(home_dir)?;
            return Ok(());
        }

        let path = Path::new(dir);
        if path.exists() {
            env::set_current_dir(path)?;
        } else {
            let no_file_fail = format!("cd: {}: No such file or directory\n", dir);
            match redirect {
                Redirect::Stderr => {
                    let buffer = buf.expect("If redirecting we should have a file buffer");
                    buffer.write_all(no_file_fail.as_bytes())?;
                }
                _ => print!("{no_file_fail}"),
            }
        }
    } else {
        let no_file_passed_in = "No file or directory passed into cd\n";
        match redirect {
            Redirect::Stderr => {
                let buffer = buf.expect("If redirecting we should have a file buffer");
                buffer.write_all(no_file_passed_in.as_bytes())?;
            }
            _ => print!("{no_file_passed_in}"),
        }
    }
    Ok(())
}
