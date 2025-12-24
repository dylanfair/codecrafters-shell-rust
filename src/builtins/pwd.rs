use std::env;

use anyhow::Result;

pub fn pwd_fn() -> Result<()> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display());
    Ok(())
}
