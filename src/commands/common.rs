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
  text    PT_LOAD    ;
  data    PT_LOAD    ;
  dynamic PT_DYNAMIC ;
}

SECTIONS
{
  . = SIZEOF_HEADERS;
  .text    : { *(.text*)   } : text    
  .rodata  : { *(.rodata*) } : text    
  .dynamic : { *(.dynamic) } : dynamic 
  .dynsym  : { *(.dynsym)  } : data    
  /DISCARD/ : {
    *(.eh_frame*)
    *(.gnu.hash*)
    *(.hash*)    
    *(.comment)  
    *(.symtab)   
    *(.strtab)   
  }
}

ENTRY (entrypoint)"#;

pub const README: &str = r#"# default_project_name

Created with [sbpf](https://github.com/deanmlittle/sbpf)"#;

pub const GITIGNORE: &str = r#"build/**/*
deploy/**/*
node_modules
.sbpf
.DS_Store
.vscode
keypair.json
package-lock.json
test-ledger
yarn.lock"#;

pub const PACKAGE_JSON: &str = r#"{
  "name": "default_project_name",
  "description": "Created with sBPF",
  "version": "1.0.0",
  "main": "index.js",
  "license": "MIT",
  "scripts": {
    "test": "cross-env SIGNER=$(cat ~/.config/solana/id.json) mocha --import=tsx tests/**/*.ts"
  },
  "dependencies": {
    "@solana/web3.js": "^1.91.8"
  },
  "devDependencies": {
    "cross-env": "^7.0.3",
    "@types/chai": "^4.3.16",
    "@types/mocha": "^10.0.6",
    "chai": "^5.1.1",
    "mocha": "^10.4.0",
    "tsx": "^4.11.0"
  }
}
"#;

pub const TESTS: &str = r#"
import { Connection, Keypair, Transaction, TransactionInstruction } from "@solana/web3.js"
import programSeed from "../deploy/default_project_name-keypair.json"

const programKeypair = Keypair.fromSecretKey(new Uint8Array(programSeed))
const program = programKeypair.publicKey
const signerSeed = JSON.parse(process.env.SIGNER!)
const signer = Keypair.fromSecretKey(new Uint8Array(signerSeed))

const connection = new Connection("http://127.0.0.1:8899", {
    commitment: "confirmed"
})

const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash()
    await connection.confirmTransaction({
        signature,
        ...block,
    })
    return signature
}

const log = async (signature: string): Promise<string> => {
    console.log(`Transaction successful! https://explorer.solana.com/tx/${signature}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899`)
    return signature
}

const signAndSend = async(tx: Transaction): Promise<string> => {
    const block = await connection.getLatestBlockhash()
    tx.recentBlockhash = block.blockhash
    tx.lastValidBlockHeight = block.lastValidBlockHeight
    const signature = await connection.sendTransaction(tx, [signer])
    return signature
}

describe('hello solana tests', () => {
    it('Logs out "Hello, Solana!"', async () => {
        const tx = new Transaction()
        tx.instructions.push(
            new TransactionInstruction({
            keys: [{
                pubkey: signer.publicKey,
                isSigner: true,
                isWritable: true
            }],
            programId: program
        }))
        await signAndSend(tx).then(confirm).then(log);
    });
});
"#;

pub const TSCONFIG: &str = r#"
{
    "compilerOptions": {
        "target": "es6",
        "module": "commonjs",
        "strict": true,
        "esModuleInterop": true,
        "resolveJsonModule": true,
        "skipLibCheck": true,
        "forceConsistentCasingInFileNames": true
    }
}
"#;
