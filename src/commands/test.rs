use anyhow::{Error, Result};
use std::{io, path::Path, process::Command};

pub fn test() -> Result<(), Error> {
    println!("🧪 Running tests");

    let has_cargo = Path::new("Cargo.toml").exists();
    let has_package_json = Path::new("package.json").exists();

    match (has_cargo, has_package_json) {
        (true, _) => {
            let output = Command::new("cargo")
                .arg("test-sbf")
                .env("RUST_BACKTRACE", "1")
                .status()?;

            if !output.success() {
                eprintln!("Failed to run Rust tests");
                return Err(Error::new(io::Error::new(
                    io::ErrorKind::Other,
                    "❌ Rust tests failed",
                )));
            }
        }
        (false, true) => {
            let status = Command::new("yarn").arg("test").status()?;

            if !status.success() {
                eprintln!("Failed to run tests");
                return Err(Error::new(io::Error::new(
                    io::ErrorKind::Other,
                    "❌ Test failed",
                )));
            }
        }
        (false, false) => {
            return Err(Error::new(io::Error::new(
                io::ErrorKind::NotFound,
                "❌ No test configuration found. Expected either Cargo.toml or package.json",
            )));
        }
    }

    println!("✅ Tests completed successfully!");
    Ok(())
}
