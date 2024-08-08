use std::fs;
use std::path::Path;

use anyhow::{Error, Result};

pub fn clean() -> Result<(), Error> {
    let deploy_path = Path::new("deploy");

    for entry in deploy_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if ext == "so" || ext == "o" {
                    if let Some(filename) = path.file_stem().and_then(|name| name.to_str()) {
                        fs::remove_file(filename)?;
                    }
                }
            }
        }
    }
    Ok(())
}
