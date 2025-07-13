use std::io;
use std::path::Path;
use std::process::Command;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

use std::fs;
use anyhow::{Error, Result};

fn deploy_program(program_name: &str, url: &str) -> Result<(), Error> {
    let deploy = "deploy";

    // Function to check if keypair file exists.
    fn has_keypair_file(dir: &Path) -> bool {
        if dir.exists() && dir.is_dir() {
            match fs::read_dir(dir) {
                Ok(entries) => entries.filter_map(Result::ok).any(|entry| {
                    entry
                        .path()
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.ends_with("-keypair.json"))
                        .unwrap_or(false)
                }),
                Err(_) => false,
            }
        } else {
            false
        }
    }

    // Check if keypair file exists. If not, create one.
    let deploy_path = Path::new(deploy);
    if !has_keypair_file(deploy_path) {
        let project_path = std::env::current_dir()?;
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("program");
        let mut rng = OsRng;
        fs::write(
            deploy_path.join(format!("{}-keypair.json", project_name)),
            serde_json::json!(SigningKey::generate(&mut rng).to_keypair_bytes()[..]).to_string(),
        )?;
    }

    let program_id_file = format!("./deploy/{}-keypair.json", program_name);
    let program_file = format!("./deploy/{}.so", program_name);

    if Path::new(&program_file).exists() {
        println!("🔄 Deploying \"{}\"", program_name);

        let status = Command::new("solana")
            .arg("program")
            .arg("deploy")
            .arg(&program_file)
            .arg("--program-id")
            .arg(&program_id_file)
            .arg("-u")
            .arg(url)
            .status()?;

        if !status.success() {
            eprintln!("Failed to deploy program for {}", program_name);
            return Err(Error::new(io::Error::new(
                io::ErrorKind::Other,
                "❌ Deployment failed",
            )));
        }

        println!("✅ \"{}\" deployed successfully!", program_name);
    } else {
        eprintln!("Program file {} not found", program_file);
        return Err(Error::new(io::Error::new(
            io::ErrorKind::NotFound,
            "❌ Program file not found",
        )));
    }

    Ok(())
}

fn deploy_all_programs(url: &str) -> Result<(), Error> {
    let deploy_path = Path::new("deploy");

    for entry in deploy_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("so") {
            if let Some(filename) = path.file_stem().and_then(|name| name.to_str()) {
                deploy_program(filename, url)?;
            }
        }
    }

    Ok(())
}

pub fn deploy(name: Option<String>, url: Option<String>) -> Result<(), Error> {
    let url = url.unwrap_or_else(|| "localhost".to_string());

    if let Some(program_name) = name {
        deploy_program(&program_name, &url)
    } else {
        deploy_all_programs(&url)
    }
}
