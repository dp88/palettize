# palettize

A command-line utility for applying ordered Bayer dithering to images with custom color palettes.

Palettize converts full-color images to a limited color palette using ordered dithering, creating the characteristic cross-hatch patterns seen in classic video games and pixel art.

## Installation

### From source

```bash
git clone https://github.com/dp88/palettize.git
cd palettize
cargo install --path .
```

### Build without installing

```bash
cargo build --release
# Binary will be at target/release/palettize
```

## Usage

```bash
palettize -i <INPUT> -o <OUTPUT> [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Input image path (PNG, JPEG, GIF, BMP, etc.) |
| `-o, --output <FILE>` | Output image path |
| `-p, --palette <COLORS>` | Comma-separated hex colors (e.g., `#000000,#FFFFFF`) |
| `--palette-file <FILE>` | Path to a palette file (one hex color per line) |
| `--preset <NAME>` | Use a built-in preset palette |
| `-b, --bayer-level <N>` | Bayer matrix level 0-5 (default: 2) |
| `-n, --noise <STRENGTH>` | Dither strength 0.0-2.0 (default: 1.0) |

### Preset Palettes

| Preset | Colors | Description |
|--------|--------|-------------|
| `bw` | 2 | Black and white |
| `grayscale` | 6 | Six-level grayscale |
| `rgb3bit` | 8 | 3-bit RGB (black, red, green, blue, yellow, magenta, cyan, white) |
| `gameboy` | 4 | Nintendo Game Boy green palette |
| `cga` | 16 | IBM CGA 16-color palette |

### Bayer Matrix Levels

The `--bayer-level` option controls the dithering pattern size:

| Level | Matrix Size | Effect |
|-------|-------------|--------|
| 0 | 2×2 | Very coarse, visible pattern |
| 1 | 4×4 | Coarse dithering |
| 2 | 8×8 | Balanced (default) |
| 3 | 16×16 | Fine dithering |
| 4 | 32×32 | Very fine |
| 5 | 64×64 | Extremely fine, subtle pattern |

## Examples

### Using a preset palette

```bash
# Apply Game Boy-style green palette
palettize -i photo.png -o gameboy.png --preset gameboy

# Convert to black and white
palettize -i photo.png -o bw.png --preset bw
```

### Using custom colors

```bash
# Sepia tone
palettize -i photo.png -o sepia.png -p "#2E1E0F,#6B4423,#C4A35A,#F5DEB3"

# Cyberpunk palette
palettize -i photo.png -o cyber.png -p "#0D0221,#0F084B,#26408B,#A6CFD5,#C2E7D9"
```

### Using a palette file

Create a file `my-palette.hex`:
```
// My custom palette
#1a1c2c
#5d275d
#b13e53
#ef7d57
#ffcd75
#a7f070
#38b764
#257179
```

Then run:
```bash
palettize -i photo.png -o output.png --palette-file my-palette.hex
```

### Adjusting dither parameters

```bash
# Finer dithering pattern with reduced contrast
palettize -i photo.png -o subtle.png --preset grayscale -b 3 -n 0.5

# Coarser pattern with increased contrast
palettize -i photo.png -o bold.png --preset grayscale -b 1 -n 1.5
```

## License

MIT
