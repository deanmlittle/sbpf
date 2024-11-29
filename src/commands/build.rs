use anyhow::{Error, Result};
use dirs::home_dir;
use std::fs;
use std::fs::create_dir_all;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use crate::commands::common::{SolanaConfig, DEFAULT_LINKER};

pub fn build() -> Result<()> {
    // Construct the path to the config file
    let home_dir = home_dir().expect("❌ Could not find $HOME directory");
    // Solana Config path
    let config_path = home_dir.join(".config/solana/install/config.yml");

    if !Path::new(&config_path).exists() {
        return Err(Error::msg("❌ Solana config not found. Please install the Solana CLI:\n\nhttps://docs.solanalabs.com/cli/install"));
    }

    // Read the file contents
    let config_content = fs::read_to_string(config_path)?;

    // Parse the YAML file
    let solana_config: SolanaConfig = serde_yaml::from_str(&config_content)?;

    // Solana SDK and toolchain paths
    let platform_tools = [
        solana_config.active_release_dir,
        "/bin/sdk/sbf/dependencies/platform-tools".to_owned(),
    ]
    .concat();
    let llvm_dir = [platform_tools.clone(), "/llvm".to_owned()].concat();
    let clang = [llvm_dir.clone(), "/bin/clang".to_owned()].concat();
    let ld = [llvm_dir.clone(), "/bin/ld.lld".to_owned()].concat();

    // Check for platform tools
    if !Path::new(&llvm_dir).exists() {
        return Err(Error::msg(format!("❌ Solana platform-tools not found. To manually install, please download the latest release here: \n\nhttps://github.com/anza-xyz/platform-tools/releases\n\nThen unzip to this directory and try again:\n\n{}", &platform_tools)));
    }

    // Set src/out directory and compiler flags
    let src = "src";
    let out = ".sbpf";
    let deploy = "deploy";
    let arch = "-target";
    let arch_target = "sbf";
    let march = "-march=bpfel+solana";

    // Create necessary directories
    create_dir_all(out)?;
    create_dir_all(deploy)?;

    // Function to compile assembly
    fn compile_assembly(
        clang: &str,
        arch: &str,
        arch_target: &str,
        march: &str,
        out: &str,
        src: &str,
        filename: &str,
    ) -> Result<()> {
        let output_file = format!("{}/{}.o", out, filename);
        let input_file = format!("{}/{}/{}.s", src, filename, filename);
        let status = Command::new(clang)
            .args([
                arch,
                arch_target,
                march,
                "-Os",
                "-c",
                "-o",
                &output_file,
                &input_file,
            ])
            .status()?;

        if !status.success() {
            eprintln!("Failed to compile assembly for {}", filename);
            return Err(Error::new(io::Error::new(
                io::ErrorKind::Other,
                "Compilation failed",
            )));
        }
        Ok(())
    }

    // Function to build shared object
    fn build_shared_object(ld: &str, filename: &str) -> Result<()> {
        let default_linker = ".sbpf/linker.ld".to_string();
        let output_file = format!("deploy/{}.so", filename);
        let input_file = format!(".sbpf/{}.o", filename);
        let mut linker_file = format!("src/{}.ld", filename);
        // Check if a custom linker file exists
        if !Path::new(&linker_file).exists() {
            if !Path::new(&default_linker).exists() {
                fs::create_dir(".sbpf").unwrap_or(());
                fs::write(&default_linker, DEFAULT_LINKER)?;
            }
            linker_file = default_linker;
        };

        let status = Command::new(ld)
            .arg("-shared")
            .arg("-z")
            .arg("notext")
            .arg("--image-base")
            .arg("0x100000000")
            .arg("-T")
            .arg(linker_file)
            .arg("-o")
            .arg(&output_file)
            .arg(&input_file)
            .status()?;

        if !status.success() {
            eprintln!("Failed to build shared object for {}", filename);
            return Err(Error::new(io::Error::new(
                io::ErrorKind::Other,
                "Linking failed",
            )));
        }
        Ok(())
    }

    // Processing directories
    let src_path = Path::new(src);
    for entry in src_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(subdir) = path.file_name().and_then(|name| name.to_str()) {
                let asm_file = format!("{}/{}/{}.s", src, subdir, subdir);
                if Path::new(&asm_file).exists() {
                    println!("🔄 Building \"{}\"", subdir);
                    let start = Instant::now();
                    compile_assembly(&clang, arch, arch_target, march, out, src, subdir)?;
                    build_shared_object(&ld, subdir)?;
                    let duration = start.elapsed();
                    println!(
                        "✅ \"{}\" built successfully in {}ms!",
                        subdir,
                        duration.as_micros() as f64 / 1000.0
                    );
                }
            }
        }
    }

    Ok(())
}
