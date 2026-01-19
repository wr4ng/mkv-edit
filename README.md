# mkv-edit

My try at implementing an `.mkv` parser and editor using Rust.

The purpose is to learn more about the Matroska format and binary parsing using Rust,
not to create a tool competing with tools like `mkvtoolnix`.

# Usage
The code is very much in development.
The `/examples` can be used to parse EBML or `.mkv` files with current functionality:
```shell
cargo run --example=read_ebml -- sample.mkv         # Parses and prints EBML structure (debug)
cargo run --example=parse_matroska -- sample.mkv    # Parses and print Matroska document structure
```
