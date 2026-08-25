# palettize-cli

[![crates.io](https://img.shields.io/crates/v/palettize-cli.svg)](https://crates.io/crates/palettize-cli)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

`palettize-cli` installs `palettize`, a command-line tool for ordered Bayer dithering with custom palettes.

## Quick start

```sh
cargo install palettize-cli
palettize --input photo.png --output photo-dithered.png --grayscale 6
```

From a checkout:

```sh
cargo install --path crates/palettize-cli
```

Run `palettize --help` for palette, extraction, Bayer-matrix, and dither-strength options.

## Requirements

Building from source requires Rust 1.85 or later.

## License

Licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
