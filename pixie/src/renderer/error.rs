use std::fmt;

use crate::AtlasError;

#[derive(Debug)]
pub enum FontError {
    InvalidFont(&'static str),
    AtlasTooSmall {
        atlas_size: usize,
        glyph_width: usize,
        glyph_height: usize,
    },
}

impl fmt::Display for FontError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFont(message) => write!(formatter, "failed to load font: {message}"),
            Self::AtlasTooSmall {
                atlas_size,
                glyph_width,
                glyph_height,
            } => write!(
                formatter,
                "font glyph cell {glyph_width}x{glyph_height} does not fit in {atlas_size}x{atlas_size} atlas"
            ),
        }
    }
}

impl std::error::Error for FontError {}

#[derive(Debug)]
pub enum RenderError {
    Atlas(AtlasError),
    Font(FontError),
    MissingGpuResource {
        resource_type: &'static str,
        name: String,
    },
    Surface(wgpu::SurfaceError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atlas(error) => error.fmt(formatter),
            Self::Font(error) => error.fmt(formatter),
            Self::MissingGpuResource {
                resource_type,
                name,
            } => {
                write!(formatter, "GPU {resource_type} '{name}' is not initialized")
            }
            Self::Surface(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Atlas(error) => Some(error),
            Self::Font(error) => Some(error),
            Self::MissingGpuResource { .. } => None,
            Self::Surface(error) => Some(error),
        }
    }
}

impl From<AtlasError> for RenderError {
    fn from(value: AtlasError) -> Self {
        Self::Atlas(value)
    }
}

impl From<FontError> for RenderError {
    fn from(value: FontError) -> Self {
        Self::Font(value)
    }
}

impl From<wgpu::SurfaceError> for RenderError {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::Surface(value)
    }
}
