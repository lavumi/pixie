use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use hecs::Entity;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AtlasId(Arc<str>);

impl AtlasId {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for AtlasId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AtlasId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for AtlasId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone)]
enum TextureAtlasBytes {
    Static(&'static [u8]),
    Owned(Arc<[u8]>),
}

#[derive(Clone)]
pub struct TextureAtlasAsset {
    id: AtlasId,
    bytes: TextureAtlasBytes,
}

impl TextureAtlasAsset {
    pub fn from_static(id: impl Into<AtlasId>, bytes: &'static [u8]) -> Self {
        Self {
            id: id.into(),
            bytes: TextureAtlasBytes::Static(bytes),
        }
    }

    pub fn from_owned(id: impl Into<AtlasId>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            id: id.into(),
            bytes: TextureAtlasBytes::Owned(Arc::from(bytes.into())),
        }
    }

    pub fn id(&self) -> &AtlasId {
        &self.id
    }

    pub fn bytes(&self) -> &[u8] {
        match &self.bytes {
            TextureAtlasBytes::Static(bytes) => bytes,
            TextureAtlasBytes::Owned(bytes) => bytes,
        }
    }
}

#[derive(Debug)]
pub enum AtlasError {
    DuplicateAtlas {
        atlas: AtlasId,
    },
    MissingAtlas {
        atlas: AtlasId,
        entity: Entity,
    },
    InvalidAtlasImage {
        atlas: AtlasId,
        source: image::ImageError,
    },
    MissingGpuAtlas {
        atlas: AtlasId,
    },
}

impl fmt::Display for AtlasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAtlas { atlas } => {
                write!(formatter, "texture atlas '{atlas}' is already registered")
            }
            Self::MissingAtlas { atlas, entity } => {
                write!(
                    formatter,
                    "entity {entity:?} references unregistered texture atlas '{atlas}'"
                )
            }
            Self::InvalidAtlasImage { atlas, source } => {
                write!(
                    formatter,
                    "failed to decode texture atlas '{atlas}': {source}"
                )
            }
            Self::MissingGpuAtlas { atlas } => {
                write!(
                    formatter,
                    "texture atlas '{atlas}' is not loaded on the GPU"
                )
            }
        }
    }
}

impl std::error::Error for AtlasError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidAtlasImage { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct TextureAtlasRegistry {
    registered: HashSet<AtlasId>,
    loaded: HashSet<AtlasId>,
    pending: VecDeque<TextureAtlasAsset>,
    pending_error: Option<AtlasError>,
}

impl TextureAtlasRegistry {
    pub fn register(&mut self, asset: TextureAtlasAsset) -> Result<(), AtlasError> {
        let atlas = asset.id().clone();
        if !self.registered.insert(atlas.clone()) {
            self.pending_error = Some(AtlasError::DuplicateAtlas {
                atlas: atlas.clone(),
            });
            return Err(AtlasError::DuplicateAtlas { atlas });
        }
        self.pending.push_back(asset);
        Ok(())
    }

    pub fn is_loaded(&self, atlas: &AtlasId) -> bool {
        self.loaded.contains(atlas)
    }

    pub(crate) fn take_pending(&mut self) -> Vec<TextureAtlasAsset> {
        self.pending.drain(..).collect()
    }

    pub(crate) fn take_error(&mut self) -> Option<AtlasError> {
        self.pending_error.take()
    }

    pub(crate) fn mark_loaded(&mut self, atlas: AtlasId) {
        self.loaded.insert(atlas);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn atlas_id_clone_preserves_identity() {
        let atlas = AtlasId::from("player");
        let clone = atlas.clone();
        let ids = HashSet::from([atlas]);

        assert!(ids.contains(&clone));
        assert_eq!(clone.as_str(), "player");
    }

    #[test]
    fn duplicate_pending_or_loaded_atlas_is_rejected() {
        let mut registry = TextureAtlasRegistry::default();
        registry
            .register(TextureAtlasAsset::from_static("player", b"first"))
            .unwrap();

        let duplicate = registry
            .register(TextureAtlasAsset::from_owned("player", b"second".to_vec()))
            .unwrap_err();
        assert!(matches!(duplicate, AtlasError::DuplicateAtlas { .. }));
        assert!(matches!(
            registry.take_error(),
            Some(AtlasError::DuplicateAtlas { .. })
        ));

        let asset = registry.take_pending().pop().unwrap();
        registry.mark_loaded(asset.id().clone());
        let duplicate = registry
            .register(TextureAtlasAsset::from_static("player", b"third"))
            .unwrap_err();
        assert!(matches!(duplicate, AtlasError::DuplicateAtlas { .. }));
    }

    #[test]
    fn static_and_owned_assets_expose_bytes() {
        let static_asset = TextureAtlasAsset::from_static("static", b"static");
        let owned_asset = TextureAtlasAsset::from_owned("owned", b"owned".to_vec());

        assert_eq!(static_asset.bytes(), b"static");
        assert_eq!(owned_asset.bytes(), b"owned");
    }
}
