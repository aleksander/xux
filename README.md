# Haven and Hearth MMORPG alternative client and tools #

This repo contains:

* several clients for Haven and Hearth (havenandhearth.com) MMORPG
* hafen protocol parser
* stapler tool to staple map tiles to one big picture

## Prerequisites ##

* Rust compiler
* Cargo package manager
* Registered account at havenandhearth.com

## Building and running ##

To build and run a client do:
```
cargo run --release -p client-macroquad -- <USERNAME> <PASSWORD>
```
To build and run parser tool do:
```
cargo run --release -p parser -- <PCAP>
```
To build and run stapler tool do:
```
cargo run --release -p stapler
```
