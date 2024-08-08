use std::env;
use std::fs::create_dir_all;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::{Error, Result};

pub fn build() -> Result<()> {
    // Solana SDK and toolchain paths
    let home_dir = env::var("HOME").expect("Could not find home directory");
    let solana_sdk = format!(
        "{}/.local/share/solana/install/active_release/bin/sdk/sbf/dependencies",
        home_dir
    );
    let llvm_dir = format!("{}/platform-tools/llvm", solana_sdk);
    let clang = format!("{}/bin/clang", llvm_dir);
    let ld = format!("{}/bin/ld.lld", llvm_dir);

    // Set src/out directory and compiler flags
    let src = "src";
    let out = "build";
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
    fn build_shared_object(
        ld: &str,
        out: &str,
        src: &str,
        deploy: &str,
        subdir: &str,
        filename: &str,
    ) -> Result<()> {
        let output_file = format!("{}/{}.so", deploy, filename);
        let input_file = format!("{}/{}.o", out, filename);
        let linker_script = format!("{}/{}/{}.ld", src, subdir, filename);
        let status = Command::new(ld)
            .arg("-shared")
            .arg("-z")
            .arg("notext")
            .arg("--image-base")
            .arg("0x100000000")
            .arg("-T")
            .arg(&linker_script)
            .arg("-o")
            .arg(&output_file)
            .arg(&input_file)
            .status()?;

        if !status.success() {
            eprintln!("Failed to build shared object for {}", filename);
            return Err(Error::new(io::Error::new(io::ErrorKind::Other, "Linking failed")));
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
                    build_shared_object(&ld, out, src, deploy, subdir, subdir)?;
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
