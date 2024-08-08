use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use super::defaults::{BUILD_SCRIPT, DEFAULT_LINKER, DEFAULT_PROGRAM, GITIGNORE, PACKAGE_JSON, README, TSCONFIG};
use anyhow::{Error, Result};
use solana_sdk::signature::Keypair;

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
        fs::create_dir(project_path)?;
        fs::create_dir(project_path.join("src"))?;
        fs::create_dir(project_path.join("src").join(&project_name))?;
        fs::create_dir(project_path.join("deploy"))?;
        fs::create_dir(project_path.join("build"))?;
        fs::create_dir(project_path.join("tests"))?;

        // Create Readme
        fs::write(project_path.join("README.md"), README.replace("default_project_name", &project_name))?;
        // Create .gitignore
        fs::write(project_path.join(".gitignore"), GITIGNORE)?;
        // Create linker file
        fs::write(project_path.join(format!("src/{}/{}.ld", project_name, project_name)), DEFAULT_LINKER)?;
        // Create default program
        fs::write(project_path.join(format!("src/{}/{}.s", project_name, project_name)), DEFAULT_PROGRAM)?;
        // Create deploy keypair
        fs::write(project_path.join("deploy").join(format!("{}-keypair.json", project_name)), serde_json::json!(Keypair::new().to_bytes()[..]).to_string())?;
        // Create package.json
        fs::write(project_path.join("package.json"), PACKAGE_JSON.replace("default_project_name", &project_name))?;
        // Create tsconfig.json
        fs::write(project_path.join("tsconfig.json"), TSCONFIG)?;
        // Create build script and set executable permission     
        fs::write(project_path.join("build.sh"), BUILD_SCRIPT)?;
        fs::set_permissions(&project_path.join("build.sh"), fs::Permissions::from_mode(0o755))?;
        println!("✅ Project '{}' initialized successfully!", project_name);
        Ok(())
    } else {
        println!("⚠️ Project '{}' already exists!", project_name);
        Ok(())
    }
}
