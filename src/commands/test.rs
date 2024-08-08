use std::io;
use std::process::Command;

use anyhow::{Error, Result};

pub fn test() -> Result<(), Error> {
    println!("🧪 Running tests");

    let status = Command::new("yarn")
        .arg("test")
        .status()?;

    if !status.success() {
        eprintln!("Failed to run tests");
        return Err(Error::new(io::Error::new(
            io::ErrorKind::Other,
            "❌ Test failed",
        )));
    }
    Ok(())
}