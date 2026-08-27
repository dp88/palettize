# Changelog

All notable changes to this project are documented in this file.

## Unreleased

### Added

- `palettize::remap_to_palette()` matches an image's color distribution to a palette's distribution in Oklab before dithering.
- `--remap [STRENGTH]` on the CLI applies the remap, with an optional strength that defaults to 1.0.
