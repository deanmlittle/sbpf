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
test-ledger
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
