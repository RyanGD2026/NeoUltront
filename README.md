# NeoUltront

NeoUltront is a Rust-based font editor project. The current work establishes its native project model and command-line foundation.

## Version

**0.8.0**

## Native project format

NeoUltront's editable project format is **`.nufont`**. Version 1 of the format is JSON-based and stores:

- Font family/style metadata and vertical metrics
- Glyph names, Unicode assignments, advance widths and side bearings
- Glyph outlines as move, line, quadratic, cubic and close path commands
- Kerning pairs and ligature mappings
- Basic editor settings

The format is versioned independently from the application so it can evolve while remaining identifiable.

## Requirements

- Rust 2024 toolchain
- Cargo

## Build and test

```sh
cargo build
cargo test
```

## Current commands

Create a project:

```sh
cargo run -- new MyFont.nufont "My Font"
```

Inspect a project:

```sh
cargo run -- inspect MyFont.nufont
```

Validate a project:

```sh
cargo run -- validate MyFont.nufont
```

Show help:

```sh
cargo run -- help
```

## Development status

The project model, JSON persistence, validation, and basic CLI are implemented. The graphical glyph editor, font shaping/preview, import pipeline, and TTF/OTF/WOFF2 export are not implemented yet.
