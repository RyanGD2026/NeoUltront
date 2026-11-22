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
- Dynamic glyph selection and new-glyph creation
- Click-to-add outline points on the glyph canvas
- Quadratic and cubic Bézier segment tools
- Visible Bézier control-handle guide lines
- Click-to-select existing anchor points and Bézier control handles
- Drag selected anchors and control handles directly on the glyph canvas
- Live selected-node highlight while editing
- Close-contour and undo-last-command controls
- Live SVG-path rendering of the selected glyph
- Editable family-name field
- Font metric overview (units per em, ascender, descender)
- Native open/save dialogs for `.nufont` files
- Export the selected glyph outline to a standalone SVG file
- Status messages for project operations

The glyph canvas now supports basic polygon-outline construction by clicking points, closing contours, and undoing the last outline command. Zoom/pan, multi-selection, keyboard editing shortcuts, and live font shaping are not implemented yet. Bézier curves can be drawn with the Quadratic and Cubic tools by clicking the required control points and endpoint; existing anchors and handles can be selected and dragged.

## CLI commands

```sh
cargo run -- new MyFont.nufont "My Font"
cargo run -- inspect MyFont.nufont
cargo run -- validate MyFont.nufont
```

## Development status

The project model, `.nufont` JSON persistence, validation, desktop shell, project workflow, and first point-based outline editor are in place. Font import, direct node/handle dragging, shaping/preview, and compiled TTF/OTF/WOFF2 font export remain future milestones. Individual glyphs can currently be exported as SVG.
