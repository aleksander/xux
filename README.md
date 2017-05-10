# Salem (salemthegame.com) and "Hafen" (havenandhearth.com) MMORPGs alternative client and tools #

## Prerequisites ##

- Rust compiler
- Cargo package manager

## Building and running ##

To build and run Salem flavored version of client do:
```
cargo run --release --features salem --bin xux -- [USERNAME] [PASSWORD]
```
To build and run Hafen flavored version of client do:
```
cargo run --release --features hafen --bin xux -- [USERNAME] [PASSWORD]
```
To build and run stapler app do:
```
cargo run --release --bin stapler
```
