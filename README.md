# sbpf

A simple scaffold to bootstrap sBPF Assembly programs.

### Dependencies

Please ensure you have install the latest [Solana Command Line Tools](https://docs.solanalabs.com/cli/install)

### Installation
```sh
cargo install --git https://github.com/deanmlittle/sbpf.git
```

### Usage
To view all the commands you can run, type `sbpf help`


```sh
Usage: sbpf <COMMAND>

Commands:
  init    Create a new project scaffold
  build   Compile into a Solana program executable
  deploy  Build and deploy the program
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### TODO

1. tests - Include some typescript tests to help users get started

PRs welcome!