# NeoUltront

NeoUltront is a Rust-based font editor. Version **0.8.0** introduces the first desktop application shell alongside the native font-project model.

## Technology

- Rust 2024
- [Slint](https://slint.dev/) for the native desktop interface
- `rfd` for native file dialogs
- Serde + JSON for the current `.nufont` project serialization

## Native project format

NeoUltront's editable project format is **`.nufont`**. Format version 1 stores:

- Font family/style metadata and vertical metrics
- Glyph names, Unicode assignments, advance widths and side bearings
- Glyph outlines as move, line, quadratic, cubic and close path commands
- Kerning pairs and ligature mappings
- Basic editor settings

The project format is versioned independently from the application.

## Build and run

Requirements: Rust toolchain with Cargo. The current Slint release requires a recent Rust toolchain; see the [Slint Rust documentation](https://docs.slint.dev/latest/docs/rust/slint/).

```sh
cargo build
cargo run
cargo test
```

## Current desktop features

- Dark three-panel font-editor workspace
- Dynamic glyph list populated from the opened project
- Glyph selection and new-glyph creation
- Editable family-name field
- Font metric overview (units per em, ascender, descender)
- Native open/save dialogs for `.nufont` files
- Status messages for project operations

The center glyph area is currently a visual preview placeholder. Direct vector-outline editing, node manipulation, zoom/pan tools, and live font rendering are not implemented yet.

## CLI commands

```sh
cargo run -- new MyFont.nufont "My Font"
cargo run -- inspect MyFont.nufont
cargo run -- validate MyFont.nufont
```

## Development status

The project model, `.nufont` JSON persistence, validation, desktop shell, and basic project workflow are in place. Font import, direct glyph-outline editing, shaping/preview, and TTF/OTF/WOFF2 export remain future milestones.
