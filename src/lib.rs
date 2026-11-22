//! Core library for NeoUltront.

pub mod project;

pub use project::{
    FontMetadata, Glyph, GlyphCommand, GlyphPoint, KerningPair, Ligature,
    NeoFontProject, ProjectError, ProjectSettings, PROJECT_FORMAT_VERSION,
};

/// Current NeoUltront application version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Native editable project-file extension, without the leading dot.
pub const PROJECT_EXTENSION: &str = "nufont";
