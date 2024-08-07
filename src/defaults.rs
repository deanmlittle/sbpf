
pub const DEFAULT_PROGRAM: &str = r#".globl entrypoint
entrypoint:
    lddw r1, message
    lddw r2, 14
    call sol_log_
    exit
.extern sol_log_
.rodata
    message: .ascii "Hello, Solana!"
"#;

pub const DEFAULT_LINKER: &str = r#"PHDRS
{
  text    PT_LOAD    ; # Contains our executable code and constants
  data    PT_LOAD    ; # Contains writable data
  dynamic PT_DYNAMIC ; # Used for dynamic linking at runtime
}

SECTIONS
{
  . = SIZEOF_HEADERS;
  .text    : { *(.text*)   } : text     # Executable code
  .rodata  : { *(.rodata*) } : text     # Read-only data
  .dynamic : { *(.dynamic) } : dynamic  # Dynamic linking information
  .dynsym  : { *(.dynsym)  } : data     # Dynamic linking symbol table
  /DISCARD/ : {
    *(.eh_frame*) # Exception handling frame information
    *(.gnu.hash*) # GNU-style hash tables
    *(.hash*)     # Any other type of hash tables
    *(.comment)   # Comments
    *(.symtab)    # Program symbol table
    *(.strtab)    # Program string table
  }
}

ENTRY (entrypoint) # Symbol name of our entrypoint"#;

pub const README: &str = r#"# default_project_name

Created with [sbpf](https://github.com/deanmlittle/sbpf)"#;

pub const GITIGNORE: &str = r#"build/**/*
deploy/**/*
node_modules
.DS_Store
.vscode
keypair.json
package-lock.json
yarn.lock"#;

pub const PACKAGE_JSON: &str = r#"
{
    "name": "default_project_name",
    "version": "1.0.0",
    "scripts": {
        "test": "echo \"Error: no test specified\" && exit 1"
    },
    "dependencies": {}
}
"#;

pub const TSCONFIG: &str = r#"
{
    "compilerOptions": {
        "target": "es6",
        "module": "commonjs",
        "strict": true,
        "esModuleInterop": true,
        "skipLibCheck": true,
        "forceConsistentCasingInFileNames": true
    }
}
"#;

pub const BUILD_SCRIPT: &str = r#"#!/bin/bash

# Solana SDK and toolchain paths
SOLANA_SDK="${HOME}/.local/share/solana/install/active_release/bin/sdk/sbf/dependencies"
LLVM_DIR="${SOLANA_SDK}/platform-tools/llvm"
CLANG="${LLVM_DIR}/bin/clang"
LD="${LLVM_DIR}/bin/ld.lld"

# Set src/out directory and compiler flags
SRC="src"
OUT="build"
DEPLOY="deploy"
ARCH="-target" 
ARCH_TARGET="sbf"
MARCH="-march=bpfel+solana"
LDFLAGS="-shared -z notext --image-base 0x100000000"

# Create necessary directories
mkdir -p $OUT
mkdir -p $DEPLOY

# Function to compile assembly
compile_assembly() {
    local subdir=$1
    local filename=$2

    mkdir -pv "$OUT"
    "$CLANG" $ARCH $ARCH_TARGET $MARCH -Os -c -o "${OUT}/${filename}.o" "${SRC}/${subdir}/${filename}.s"
}

# Function to calculate elapsed time in milliseconds
calculate_elapsed_time() {
    local start_sec=$1
    local start_nsec=$2
    local end_sec=$3
    local end_nsec=$4

    local total_start_nsec=$((start_sec * 1000000000 + start_nsec))
    local total_end_nsec=$((end_sec * 1000000000 + end_nsec))
    local elapsed_nsec=$((total_end_nsec - total_start_nsec))
    local elapsed_ms=$((elapsed_nsec / 1000000))

    echo "$elapsed_ms"
}

# Function to build shared object
build_shared_object() {
    local subdir=$1
    local filename=$2
    "$LD" $LDFLAGS -T "${SRC}/${subdir}/${filename}.ld" -o "${DEPLOY}/${filename}.so" "${OUT}/${filename}.o"
}

# Default target function

for dir in ${SRC}/*/; do
    dir=${dir%*/}
    subdir=$(basename $dir)

    if [ -f "${SRC}/${subdir}/${subdir}.s" ]; then
        echo "🔄 Building \"${subdir}\""
        start_s=$(date +%s)
        compile_assembly $subdir $subdir
        build_shared_object $subdir $subdir
        end_s=$(date +%s)
        elapsed_s=$((end_s - start_s))
        echo "✅ \"${subdir}\" built successfully in ${elapsed_s}s!"
    fi
done"#;