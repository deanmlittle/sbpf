use sbpf_assembler::assemble;

use anyhow::Result;
use std::path::Path;
use std::time::Instant;
use std::fs::create_dir_all;

pub fn light_build() -> Result<()> {
    // Set src/out directory
    let src = "src";
    let deploy = "deploy";

    // Create necessary directories
    create_dir_all(deploy)?;

    // Function to compile assembly
    fn compile_assembly(src: &str, deploy: &str) -> Result<()> {
        assemble(src, deploy)
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
                    println!("⚡️ Light building \"{}\"", subdir);
                    let start = Instant::now();
                    compile_assembly(&asm_file, deploy)?;
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
