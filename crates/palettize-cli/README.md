# palettize-cli

[![CI](https://github.com/dp88/palettize/actions/workflows/ci.yml/badge.svg)](https://github.com/dp88/palettize/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/palettize-cli.svg)](https://crates.io/crates/palettize-cli)
![MSRV](https://img.shields.io/badge/rust-1.85%2B-blue)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

`palettize-cli` installs `palettize`, a command-line tool for ordered Bayer dithering with custom palettes.

## Quick start

```sh
cargo install palettize-cli
palettize --input photo.png --output photo-dithered.png --grayscale 6
```

From a repository checkout, run `cargo install --path crates/palettize-cli`
at the workspace root instead.

Run `palettize --help` for palette, extraction, remap, Bayer-matrix, and
dither-strength options.

## Requirements

Building from source requires Rust 1.85 or later, edition 2024.

## More examples and documentation

- [Library API](https://docs.rs/palettize) — the [`palettize`](https://crates.io/crates/palettize) crate behind this tool.
- [Changelog](https://github.com/dp88/palettize/blob/master/CHANGELOG.md)
- [Issue tracker](https://github.com/dp88/palettize/issues)

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
