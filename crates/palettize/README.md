# palettize

[![CI](https://github.com/dp88/palettize/actions/workflows/ci.yml/badge.svg)](https://github.com/dp88/palettize/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/palettize.svg)](https://crates.io/crates/palettize)
[![docs.rs](https://img.shields.io/docsrs/palettize)](https://docs.rs/palettize)
![MSRV](https://img.shields.io/badge/rust-1.85%2B-blue)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

`palettize` applies ordered Bayer dithering to images using custom color palettes.

## Quick start

```toml
[dependencies]
image = "0.25"
palettize = "0.2"
```

```rust,no_run
use palettize::{dither, grayscale};

let image = image::open("photo.png")?;
let palette = grayscale(6);
let output = dither(&image, &palette);
output.save("photo-dithered.png")?;

# Ok::<(), image::ImageError>(())
```

## Features

- Generate Bayer matrices from 2×2 through 64×64.
- Build palettes from RGB colors, hex values, or grayscale stops.
- Extract palettes with median cut or k-means++ clustering.
- Remap an image's color distribution onto a palette's distribution in Oklab
  before dithering.

## Requirements

This crate requires Rust 1.85 or later, edition 2024.

## More examples and documentation

- [API documentation](https://docs.rs/palettize) — every public type and function.
- [Changelog](https://github.com/dp88/palettize/blob/master/CHANGELOG.md)
- [Issue tracker](https://github.com/dp88/palettize/issues)

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
