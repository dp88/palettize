# Changelog

All notable changes to this project are documented in this file.

## Unreleased

- `palettize::remap_to_palette()` matches an image's color distribution to a
  palette's distribution in Oklab before dithering.
- `--remap [STRENGTH]` on the CLI applies the remap, with an optional
  strength that defaults to 1.0.

## 0.2.0 — 2026-08-25

- Split the crate into a `palettize` library and a `palettize-cli` binary
  package in one workspace.
- Median cut and k-means++ palette extraction with CLI selection.
- Grayscale palette generator replacing the fixed presets.
- Data-driven regression test matrix and release-ready package metadata.

## 0.1.0 — 2026-01-27

Initial release.

- Ordered Bayer dithering CLI with custom color palettes.
