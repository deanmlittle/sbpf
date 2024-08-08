# sbpf

A simple scaffold to bootstrap sBPF Assembly programs.

### Dependencies

Please make sure you have the latest version of [Solana Command Line Tools](https://docs.solanalabs.com/cli/install) installed.

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
  clean   Clean up build and deploy artifacts
  test    Build, deploy and run tests
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Advanced Usage

In some cases, you may wish to create a [custom linker file](https://github.com/deanmlittle/sbpf-asm-noop/blob/master/src/noop/noop.ld) to pull in external resources or exclude certain program sections or section headers. To use a custom linker, simply include it in the src directory with the same name as your program and it will be used to override the default linker. For example:

```
src/example/example.s
src/example/example.ld
```

### TODO

1. tests - Include some typescript tests to help users get started


### Contributing

PRs welcome!