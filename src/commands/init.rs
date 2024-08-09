use crate::commands::common::TESTS;

use super::common::{DEFAULT_PROGRAM, GITIGNORE, PACKAGE_JSON, README, TSCONFIG};
use anyhow::{Error, Result};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn init(name: Option<String>) -> Result<(), Error> {
    let project_name = match name {
        Some(name) => name,
        None => loop {
            print!("What is the name of your project? ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_string();

            if !input.is_empty() {
                break input.replace(' ', "-");
            } else {
                println!("Project name cannot be empty. Please enter a valid name.");
            }
        },
    };
    let project_path = Path::new(&project_name);

    if !project_path.exists() {
        // Create project path and subdirectories
        fs::create_dir_all(project_path.join("src").join(&project_name))?;
        fs::create_dir(project_path.join("deploy"))?;
        fs::create_dir(project_path.join("tests"))?;

        // Create Readme
        fs::write(
            project_path.join("README.md"),
            README.replace("default_project_name", &project_name),
        )?;
        // Create .gitignore
        fs::write(project_path.join(".gitignore"), GITIGNORE)?;

        // Create test
        fs::write(project_path.join(format!("tests/{}.test.ts", project_name)), TESTS.replace("default_project_name", &project_name))?;

        // Create default program
        fs::write(
            project_path.join(format!("src/{}/{}.s", project_name, project_name)),
            DEFAULT_PROGRAM,
        )?;
        // Create deploy keypair
        let mut rng = OsRng;
        fs::write(
            project_path
                .join("deploy")
                .join(format!("{}-keypair.json", project_name)),
            serde_json::json!(SigningKey::generate(&mut rng).to_keypair_bytes()[..]).to_string(),
        )?;
        // Create package.json
        fs::write(
            project_path.join("package.json"),
            PACKAGE_JSON.replace("default_project_name", &project_name),
        )?;
        // Create tsconfig.json
        fs::write(project_path.join("tsconfig.json"), TSCONFIG)?;
        println!("✅ Project '{}' initialized successfully!", project_name);
        Ok(())
    } else {
        println!("⚠️ Project '{}' already exists!", project_name);
        Ok(())
    }
}
