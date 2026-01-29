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
| `-g, --grayscale <N>` | Generate grayscale palette with N colors (2-255) |
| `-b, --bayer-level <N>` | Bayer matrix level 0-5 (default: 2) |
| `-n, --noise <STRENGTH>` | Dither strength 0.0-2.0 (default: 1.0) |

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

### Using grayscale palettes

```bash
# Convert to black and white (default)
palettize -i photo.png -o bw.png

# 6-level grayscale
palettize -i photo.png -o gray6.png -g 6

# 16-level grayscale
palettize -i photo.png -o gray16.png -g 16
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
palettize -i photo.png -o subtle.png -g 6 -b 3 -n 0.5

# Coarser pattern with increased contrast
palettize -i photo.png -o bold.png -g 6 -b 1 -n 1.5
```

## Example/Test Images
The example images used for testing palettize are in the public domain.

### 🇳🇱 A View of the Bay of Santa Margherita (Genoa), Liguria, Italy by Pieter Francis Peters

Peters (1818-1903) was a Dutch landscape painter known for his luminous Mediterranean coastal scenes and Alpine views. He traveled extensively through Italy, Switzerland, and Germany, capturing picturesque harbors and mountain landscapes with warm, atmospheric light. His works reflect the Romantic tradition of idealized nature while maintaining careful attention to topographical accuracy.

### 🇫🇷 Villa Farnese With Gardens At Caprarola (1764) by Hubert Robert

Robert (1733-1808) was a French painter celebrated for his romantic depictions of ruins and garden landscapes, earning him the nickname "Robert des Ruines." He spent eleven years in Rome where he developed his signature style of architectural capriccios and picturesque decay. After the French Revolution, he was briefly imprisoned but survived to become one of the first curators of the Louvre.

### 🇺🇸 The Departure (1837) by Thomas Cole

Cole (1801-1848) was an English-born American painter who founded the Hudson River School, the first major American art movement. His allegorical series "The Course of Empire" and "The Voyage of Life" established landscape painting as a vehicle for moral and philosophical themes. He championed the American wilderness as a subject worthy of high art during a period of rapid westward expansion.

### 🇩🇰 Spring Landscape (1893) by Peder Mørk Mønsted

Mønsted (1859-1941) was a Danish realist painter renowned for his luminous landscapes and meticulous attention to natural light. He traveled extensively throughout Europe and North Africa, painting en plein air with remarkable precision. His works are characterized by their photographic clarity and masterful depiction of water, foliage, and atmospheric conditions.
