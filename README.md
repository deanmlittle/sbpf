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
  test    Test deployed program
  clean   Clean up build and deploy artifacts
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Advanced Usage

You can override the default linker with a [custom linker file](https://github.com/deanmlittle/sbpf-asm-noop/blob/master/src/noop/noop.ld) by including it in the src directory with the same name as your program. For example:

```
src/example/example.s
src/example/example.ld
```

### Contributing

PRs welcome!