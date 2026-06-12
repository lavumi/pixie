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
    AdapterRequest(wgpu::RequestAdapterError),
    Atlas(AtlasError),
    DeviceRequest(wgpu::RequestDeviceError),
    Font(FontError),
    MissingGpuResource {
        resource_type: &'static str,
        name: String,
    },
    SurfaceCreation(wgpu::CreateSurfaceError),
    SurfaceConfiguration(&'static str),
    Surface(wgpu::SurfaceError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdapterRequest(error) => {
                write!(formatter, "failed to request GPU adapter: {error}")
            }
            Self::Atlas(error) => error.fmt(formatter),
            Self::DeviceRequest(error) => {
                write!(formatter, "failed to request GPU device: {error}")
            }
            Self::Font(error) => error.fmt(formatter),
            Self::MissingGpuResource {
                resource_type,
                name,
            } => {
                write!(formatter, "GPU {resource_type} '{name}' is not initialized")
            }
            Self::SurfaceCreation(error) => {
                write!(formatter, "failed to create rendering surface: {error}")
            }
            Self::SurfaceConfiguration(message) => {
                write!(
                    formatter,
                    "failed to configure rendering surface: {message}"
                )
            }
            Self::Surface(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AdapterRequest(error) => Some(error),
            Self::Atlas(error) => Some(error),
            Self::DeviceRequest(error) => Some(error),
            Self::Font(error) => Some(error),
            Self::MissingGpuResource { .. } => None,
            Self::SurfaceCreation(error) => Some(error),
            Self::SurfaceConfiguration(_) => None,
            Self::Surface(error) => Some(error),
        }
    }
}

impl From<wgpu::RequestAdapterError> for RenderError {
    fn from(value: wgpu::RequestAdapterError) -> Self {
        Self::AdapterRequest(value)
    }
}

impl From<AtlasError> for RenderError {
    fn from(value: AtlasError) -> Self {
        Self::Atlas(value)
    }
}

impl From<wgpu::RequestDeviceError> for RenderError {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        Self::DeviceRequest(value)
    }
}

impl From<FontError> for RenderError {
    fn from(value: FontError) -> Self {
        Self::Font(value)
    }
}

impl From<wgpu::CreateSurfaceError> for RenderError {
    fn from(value: wgpu::CreateSurfaceError) -> Self {
        Self::SurfaceCreation(value)
    }
}

impl From<wgpu::SurfaceError> for RenderError {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::Surface(value)
    }
}
