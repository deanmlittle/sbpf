use std::fs;
use std::path::Path;

use anyhow::{Error, Result};

pub fn clean() -> Result<(), Error> {
    fs::remove_dir_all(".sbpf")?;
    clean_directory("deploy", "so")?;
    Ok(())
}

fn clean_directory(directory: &str, extension: &str) -> Result<(), Error> {
    let path = Path::new(directory);
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if extension == "" || ext == extension {
                    fs::remove_file(&path)?;
                }
            }
        }
    }
    Ok(())
}