# Salem and Hafen alternative client and tools #

This repo contains:
- Two-in-one client for:
  - Hafen (havenandhearth.com) MMOG
  - Salem (salemthegame.com) MMOG
- stapler tool to staple map tiles to one big picture

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
To build and run stapler tool do:
```
cargo run --release --bin stapler
```
