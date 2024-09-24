use anyhow::{Error, Result};
use solana_rbpf::{
    elf::Executable, program::BuiltinProgram, static_analysis::Analysis, vm::TestContextObject,
};

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn disassemble(path: Option<String>, outfile: Option<String>) -> Result<()> {
    let path = path.ok_or_else(|| Error::msg("Path not provided"))?;
    let path = Path::new(&path);

    // Ensure the path points to a valid .so binary
    if path.extension().and_then(|ext| ext.to_str()) != Some("so") {
        return Err(Error::msg("Invalid file path. Expected .so file"));
    }

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let program: &'static [u8] = Box::leak(buffer.into_boxed_slice());

    let loader = Arc::new(BuiltinProgram::new_mock());
    let executable = Executable::<TestContextObject>::from_elf(program, loader).unwrap();
    let analysis = Analysis::from_executable(&executable).unwrap();
    let stdout = std::io::stdout();

    match outfile {
        Some(outpath) => {
            let outpath = ensure_asm_extension(outpath);
            let mut file = File::create(outpath)?;
            analysis.disassemble(&mut stdout.lock())?;
            analysis.disassemble(&mut file)?;
        }
        None => {
            let stdout = std::io::stdout();
            analysis.disassemble(&mut stdout.lock())?;
        }
    }

    Ok(())
}

fn ensure_asm_extension(path: String) -> PathBuf {
    let path = PathBuf::from(path);
    if path.extension().and_then(|ext| ext.to_str()) != Some("s") {
        path.with_extension("s")
    } else {
        path
    }
}
