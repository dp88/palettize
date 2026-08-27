# palettize

[![crates.io](https://img.shields.io/crates/v/palettize.svg)](https://crates.io/crates/palettize)
[![docs.rs](https://img.shields.io/docsrs/palettize)](https://docs.rs/palettize)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

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
- Remap an image's color distribution onto a palette's distribution in Oklab before dithering.

## Documentation

See the [API documentation](https://docs.rs/palettize) for every public type and function.

## Requirements

This crate requires Rust 1.85 or later.

## License

Licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
