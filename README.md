# ruxnasm

[![CI](https://github.com/karolbelina/ruxnasm/actions/workflows/ci.yml/badge.svg)](https://github.com/karolbelina/ruxnasm/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ruxnasm.svg)](https://crates.io/crates/ruxnasm)
[![docs.rs](https://docs.rs/ruxnasm/badge.svg)](https://docs.rs/ruxnasm)

:construction: Not in a useful state yet :construction:

Ruxnasm is an assembler for [Uxntal][uxntal], a programming language for the [Uxn][uxn] stack-machine by [Hundred Rabbits](https://github.com/hundredrabbits). Ruxnasm strives to be an alternative to [Uxnasm][uxnasm], featuring more user-friendly error reporting, warnings, and helpful hints, reminiscent of those seen in modern compilers for languages such as Rust or Elm.

## Quick start

```console
cargo run -- examples/helloworld.tal helloworld.rom
uxnemu helloworld.rom
```

## Compatibility with Uxnasm

Currently, Uxntal doesn't have an official language specification, which means it is defined by the programs it's processed by &mdash; the assemblers. The official assembler for Uxntal is [Uxnasm][uxnasm], written in ANSI C. Ruxnasm does not try to be a 1:1 reimplementation of Uxnasm; it's too opinionated to be so. Instead, it tries to define a more elegant and modern version of Uxntal, while at the same time preserving the software already written with Uxnasm in mind.

Although they are _mostly_ the same, there are programs that are valid in Uxnasm and invalid in Ruxnasm and vice versa. This means that the language defined by Ruxnasm is neither a subset nor a superset of the language defined by Uxnasm. All known differences between Ruxnasm and Uxnasm have been documented in the [docs/differences.md](docs/differences.md) file and are kept up-to-date as the project is being developed.

Interacting with Uxnasm from the command line is no different for Ruxnasm &mdash; just append an "r" at the start.

## Installation

### From binaries

Check out the [releases page](https://github.com/karolbelina/ruxnasm/releases) for prebuilt releases of Ruxnasm for various operating systems. If you want to get the most recent Linux, Windows, or macOS build, check out the artifacts of the latest CI workflow run on the [actions page](https://github.com/karolbelina/ruxnasm/actions).

### From source

You can build and install Ruxnasm from source using Cargo &mdash; Rust's package manager. You can get it by installing the most recent release of [Rust](https://www.rust-lang.org/). Both of the methods listed below should build the Ruxnasm binary and place it in Cargo installation root's `bin` folder (`~/.cargo/bin` as the default, check out [this guide](https://doc.rust-lang.org/cargo/commands/cargo-install.html) for more information).

- #### From the Git repository

  To build and install the most recent version of Ruxnasm, clone the repository, `cd` into it, and run
  ```console
  cargo install --path .
  ```
- #### From crates.io

  Ruxnasm can be fetched from the [crates.io](https://crates.io/crates/ruxnasm) package registry. To build and install the most recent release of Ruxnasm, run
  ```console
  cargo install ruxnasm
  ```
  from anywhere.

## Library

Besides being a command-line tool, Ruxnasm is also available as a library for the Rust programming language. It exposes the `assemble` function, which can turn a string with an Uxntal program into an Uxn binary.
```rust
pub fn assemble(source: impl AsRef<str>) -> Result<Vec<u8>>
```
The library is available on [crates.io](https://crates.io/crates/ruxnasm) and can be included in your Cargo-enabled project like this:
```toml
[dependencies]
ruxnasm = { version = "*", default-features = false } # Disable the default "bin" feature
```
and then used in your code like this:
```rust
let (binary, _) = ruxnasm::assemble("|0100 #02 #03 ADD").unwrap();

assert_eq!(binary, [0x01, 0x02, 0x01, 0x03, 0x18]);
```
The code above unwraps the result, but could just as well handle all the errors and warnings returned from the `assemble` function in case there were any.

## License

This software is licensed under the MIT license.

See the [LICENSE](LICENSE) file for more details.

[uxn]: https://wiki.xxiivv.com/site/uxn.html
[uxntal]: https://wiki.xxiivv.com/site/uxntal.html
[uxnasm]: https://git.sr.ht/~rabbits/uxn/tree/master/item/src/uxnasm.c
