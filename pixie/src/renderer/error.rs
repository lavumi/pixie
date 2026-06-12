use std::fmt;

use crate::AtlasError;

#[derive(Debug)]
pub enum RenderError {
    Atlas(AtlasError),
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

impl From<wgpu::SurfaceError> for RenderError {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::Surface(value)
    }
}
