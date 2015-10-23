A terminal based simulator of the Conway's game of life written in
Rust.


## Installation

Currently, [rust](https://www.rust-lang.org/) nigtly is required (due
to `#![feature(test)]` for benchmarks.)

```
cargo build --release
```

The binary will then be located under `./target/release/cgol`.


## Usage

Without any arguments the program generates a random world spanning
the terminal.  Alternatively, with `--file myfile.cells` specifies a
predefined world to be loaded.  The (currently only) understood format
is [plaintext](http://www.conwaylife.com/wiki/Plaintext); this is the
format the `*.cells` files in the
[pattern collection from the LifeWiki](http://www.conwaylife.com/patterns/all.zip)
are encoded.

The UI is plain simple and understands (only) the following key
strokes at the moment:

- `q` quits the program
- `s` advances the game by one generation
- `Space` starts/stops automatic advancement of the game
- `+` increases the speed of the automatic advancement
- `-` decreases the speed of the automatic advancement
- `r` regenerates a new random world


## Motivation

This program is made just for fun.  A nice way to learn and play with
Rust and [Rustbox](https://github.com/gchp/rustbox).
