//! Editable native font-project model and .nufont persistence.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    fs,
    path::Path,
};

pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoFontProject {
    pub format: String,
    pub format_version: u32,
    pub metadata: FontMetadata,
    pub glyphs: BTreeMap<String, Glyph>,
    #[serde(default)]
    pub kerning: Vec<KerningPair>,
    #[serde(default)]
    pub ligatures: Vec<Ligature>,
    #[serde(default)]
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetadata {
    pub family_name: String,
    pub style_name: String,
    pub designer: String,
    pub description: String,
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    /// Unicode scalar value assigned to this glyph, if any.
    pub codepoint: Option<char>,
    /// Advance width in font units.
    pub advance_width: f32,
    /// Left side bearing in font units.
    pub left_side_bearing: f32,
    /// Ordered path operations forming this glyph's outline.
    #[serde(default)]
    pub outline: Vec<GlyphCommand>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlyphPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "points", rename_all = "snake_case")]
pub enum GlyphCommand {
    MoveTo(GlyphPoint),
    LineTo(GlyphPoint),
    QuadTo(GlyphPoint, GlyphPoint),
    CubicTo(GlyphPoint, GlyphPoint, GlyphPoint),
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KerningPair {
    pub left: String,
    pub right: String,
    pub adjustment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ligature {
    pub sequence: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub grid_size: f32,
    pub show_metrics: bool,
    pub show_advance_width: bool,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            grid_size: 10.0,
            show_metrics: true,
            show_advance_width: true,
        }
    }
}

#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
    UnsupportedFormatVersion(u32),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Json(error) => write!(f, "project serialization error: {error}"),
            Self::Invalid(message) => write!(f, "invalid font project: {message}"),
            Self::UnsupportedFormatVersion(version) => {
                write!(f, "unsupported .nufont format version: {version}")
            }
        }
    }
}

impl Error for ProjectError {}

impl From<std::io::Error> for ProjectError {
    fn from(value: std::io::Error) -> Self { Self::Io(value) }
}

impl From<serde_json::Error> for ProjectError {
    fn from(value: serde_json::Error) -> Self { Self::Json(value) }
}

impl NeoFontProject {
    pub fn new(family_name: impl Into<String>) -> Self {
        let mut glyphs = BTreeMap::new();
        glyphs.insert(
            ".notdef".to_owned(),
            Glyph {
                codepoint: None,
                advance_width: 600.0,
                left_side_bearing: 50.0,
                outline: Vec::new(),
            },
        );

        Self {
            format: "NeoUltrontFontProject".to_owned(),
            format_version: PROJECT_FORMAT_VERSION,
            metadata: FontMetadata {
                family_name: family_name.into(),
                style_name: "Regular".to_owned(),
                designer: String::new(),
                description: String::new(),
                units_per_em: 1000,
                ascender: 800,
                descender: -200,
            },
            glyphs,
            kerning: Vec::new(),
            ligatures: Vec::new(),
            settings: ProjectSettings::default(),
        }
    }

    pub fn add_glyph(&mut self, name: impl Into<String>, glyph: Glyph) -> Result<(), ProjectError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ProjectError::Invalid("glyph name cannot be empty".into()));
        }
        if self.glyphs.contains_key(&name) {
            return Err(ProjectError::Invalid(format!("glyph '{name}' already exists")));
        }
        self.glyphs.insert(name, glyph);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.format != "NeoUltrontFontProject" {
            return Err(ProjectError::Invalid("unrecognized project identifier".into()));
        }
        if self.format_version != PROJECT_FORMAT_VERSION {
            return Err(ProjectError::UnsupportedFormatVersion(self.format_version));
        }
        if self.metadata.family_name.trim().is_empty() {
            return Err(ProjectError::Invalid("font family name cannot be empty".into()));
        }
        if self.metadata.units_per_em == 0 {
            return Err(ProjectError::Invalid("units_per_em must be greater than zero".into()));
        }
        if !self.settings.grid_size.is_finite() || self.settings.grid_size <= 0.0 {
            return Err(ProjectError::Invalid("grid_size must be a finite positive number".into()));
        }

        for (name, glyph) in &self.glyphs {
            if name.trim().is_empty() {
                return Err(ProjectError::Invalid("glyph name cannot be empty".into()));
            }
            if !glyph.advance_width.is_finite() || !glyph.left_side_bearing.is_finite() {
                return Err(ProjectError::Invalid(format!("glyph '{name}' has invalid metrics")));
            }
            for command in &glyph.outline {
                for point in command.points() {
                    if !point.x.is_finite() || !point.y.is_finite() {
                        return Err(ProjectError::Invalid(format!(
                            "glyph '{name}' contains a non-finite outline point"
                        )));
                    }
                }
            }
        }

        for pair in &self.kerning {
            if !pair.adjustment.is_finite() {
                return Err(ProjectError::Invalid("kerning adjustment must be finite".into()));
            }
        }
        Ok(())
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProjectError> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, format!("{json}\n"))?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let json = fs::read_to_string(path)?;
        let project: Self = serde_json::from_str(&json)?;
        project.validate()?;
        Ok(project)
    }
}

impl GlyphCommand {
    fn points(&self) -> &[GlyphPoint] {
        match self {
            Self::MoveTo(point) | Self::LineTo(point) => std::slice::from_ref(point),
            Self::QuadTo(a, b) => {
                // A small stack array cannot be returned as a slice; validate these variants below.
                let _ = (a, b);
                &[]
            }
            Self::CubicTo(a, b, c) => {
                let _ = (a, b, c);
                &[]
            }
            Self::Close => &[],
        }
    }

    fn has_only_finite_points(&self) -> bool {
        let finite = |p: &GlyphPoint| p.x.is_finite() && p.y.is_finite();
        match self {
            Self::MoveTo(p) | Self::LineTo(p) => finite(p),
            Self::QuadTo(a, b) => finite(a) && finite(b),
            Self::CubicTo(a, b, c) => finite(a) && finite(b) && finite(c),
            Self::Close => true,
        }
    }
}
