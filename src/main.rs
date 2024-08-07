pub mod defaults;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use clap::{Args, Parser, Subcommand};
use defaults::{BUILD_SCRIPT, DEFAULT_LINKER, DEFAULT_PROGRAM, GITIGNORE, PACKAGE_JSON, README, TSCONFIG};
use solana_sdk::signature::Keypair;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    // Deploy,
}

#[derive(Args)]
struct InitArgs {
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => {
            let project_name = match args.name.clone() {
                Some(name) => name,
                None => loop {
                    print!("What is the name of your project? ");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim().to_string();
    
                    if !input.is_empty() {
                        break input.replace(' ', "-");
                    } else {
                        println!("Project name cannot be empty. Please enter a valid name.");
                    }
                },
            };
            bootstrap(&project_name);
        }
        // Commands::Deploy => todo!(),
    }
}

fn bootstrap(project_name: &str) {
    let project_path = Path::new(project_name);

    if !project_path.exists() {
        // Create project path and subdirectories
        fs::create_dir(project_path).unwrap();
        fs::create_dir(project_path.join("src")).unwrap();
        fs::create_dir(project_path.join("src").join(project_name)).unwrap();
        fs::create_dir(project_path.join("deploy")).unwrap();
        fs::create_dir(project_path.join("build")).unwrap();
        fs::create_dir(project_path.join("tests")).unwrap();

        // Create Readme
        fs::write(project_path.join("README.md"), README.replace("default_project_name", project_name)).unwrap();
        // Create .gitignore
        fs::write(project_path.join(".gitignore"), GITIGNORE).unwrap();
        // Create linker file
        fs::write(project_path.join(format!("src/{}/{}.ld", project_name, project_name)), DEFAULT_LINKER).unwrap();
        // Create default program
        fs::write(project_path.join(format!("src/{}/{}.s", project_name, project_name)), DEFAULT_PROGRAM).unwrap();
        // Create deploy keypair
        fs::write(project_path.join("deploy").join(format!("{}-keypair.json", project_name)), serde_json::json!(Keypair::new().to_bytes()[..]).to_string()).unwrap();
        // Create package.json
        fs::write(project_path.join("package.json"), PACKAGE_JSON.replace("default_project_name", project_name)).unwrap();
        // Create tsconfig.json
        fs::write(project_path.join("tsconfig.json"), TSCONFIG).unwrap();
        // Create build script and set executable permission     
        fs::write(project_path.join("build.sh"), BUILD_SCRIPT).unwrap();
        fs::set_permissions(&project_path.join("build.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        println!("✅ Project '{}' initialized successfully!", project_name);
    } else {
        println!("⚠️ Project '{}' already exists!", project_name);
    }
}
