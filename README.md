![palettize banner](art/banner.png)

# palettize

[![CI](https://github.com/dp88/palettize/actions/workflows/ci.yml/badge.svg)](https://github.com/dp88/palettize/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/palettize.svg)](https://crates.io/crates/palettize)
[![docs.rs](https://img.shields.io/docsrs/palettize)](https://docs.rs/palettize)
![MSRV](https://img.shields.io/badge/rust-1.85%2B-blue)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

`palettize` turns full-color images into ordered Bayer-dithered images with a palette you choose.

## Quick start

Install the command-line tool:

```sh
cargo install palettize-cli
```

Then dither an image with the default black-and-white palette:

```sh
palettize --input photo.png --output photo-dithered.png
```

To work from a checkout instead, run:

```sh
cargo install --path crates/palettize-cli
```

Use `palettize --help` to see every option.

## Capabilities

- Use a grayscale palette, comma-separated colors, or a palette file.
- Extract a palette with median cut or k-means++ clustering.
- Match the image's color distribution to the palette before dithering with `--remap`.
- Control Bayer matrix size and dither strength.
- Use the reusable [`palettize`](https://crates.io/crates/palettize) library in Rust applications.

## Tuning the remap

`--remap` matches the image's color distribution to the palette before
dithering. Its strength value trades the image's own tone against the palette's
range. A higher value is not always better.

| Strength | Result |
| --- | --- |
| `0.3`-`0.5` | Keeps the image's own tone and opens up the shadows. Use this for dark or low-key images. |
| `1.0` | Matches the palette's tonal range in full. Use this for washed-out images, or when the image's colors sit far from the palette. |

The flag defaults to `1.0`. At that strength the remap reproduces the palette's
tonal distribution exactly. Most palettes spread their colors evenly across the
range, so a deliberately dark image becomes much brighter. Lower the strength to
keep the original mood:

```sh
palettize -i photo.png -o output.png --palette-file colors.hex --remap 0.5
```

## Documentation

- [Library API on docs.rs](https://docs.rs/palettize)
- [Library package guide](crates/palettize/README.md)
- [CLI package guide](crates/palettize-cli/README.md)
- [Changelog](CHANGELOG.md)
- [Release guide](RELEASING.md)
- [Issue tracker](https://github.com/dp88/palettize/issues)

## Requirements

Building from source requires Rust 1.85 or later.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

Banner: *A View of the Bay of Santa Margherita* by Pieter Francis Peters,
public domain. Full artwork and test-image credits are in [art/CREDITS.md](art/CREDITS.md).
