use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hecs::{Entity, World};

use crate::components::{Text, TextStyle, Tile, Transform};
use crate::renderer::{RenderFrame, TextRenderData, TileRenderData};
use crate::resources::ResourceContainer;

struct CachedText {
    version: u64,
    render_data: TextRenderData,
}

#[derive(Default)]
pub struct RenderWorldExtractor {
    tile_render_data: HashMap<String, Vec<TileRenderData>>,
    tile_atlases: Vec<String>,
    active_tile_atlases: HashSet<String>,
    text_cache: HashMap<Entity, CachedText>,
    text_render_buffer: Vec<TextRenderData>,
    cache_cleanup_counter: u32,
}

impl RenderWorldExtractor {
    pub fn with_capacity(tile_atlas_count: usize, text_count: usize) -> Self {
        Self {
            tile_render_data: HashMap::with_capacity(tile_atlas_count),
            tile_atlases: Vec::with_capacity(tile_atlas_count),
            active_tile_atlases: HashSet::with_capacity(tile_atlas_count),
            text_cache: HashMap::with_capacity(text_count),
            text_render_buffer: Vec::with_capacity(text_count),
            cache_cleanup_counter: 0,
        }
    }

    pub fn extract<'a>(
        &'a mut self,
        world: &World,
        resources: &ResourceContainer,
    ) -> RenderFrame<'a> {
        let camera_uniform = resources
            .get::<crate::resources::Camera>()
            .expect("Camera resource not found")
            .get_view_proj();

        self.extract_tiles(world);
        self.extract_texts(world);

        RenderFrame {
            camera_uniform,
            tile_render_data: &self.tile_render_data,
            tile_atlases: &self.tile_atlases,
            texts: &self.text_render_buffer,
        }
    }

    fn extract_tiles(&mut self, world: &World) {
        for tiles in self.tile_render_data.values_mut() {
            tiles.clear();
        }
        self.tile_atlases.clear();
        self.active_tile_atlases.clear();

        for (_entity, (transform, tile)) in world.query::<(&Transform, &Tile)>().iter() {
            if self.active_tile_atlases.insert(tile.atlas.clone()) {
                self.tile_atlases.push(tile.atlas.clone());
            }

            self.tile_render_data
                .entry(tile.atlas.clone())
                .or_default()
                .push(TileRenderData {
                    position: transform.position,
                    size: transform.size,
                    uv: tile.uv,
                });
        }
    }

    fn extract_texts(&mut self, world: &World) {
        self.text_render_buffer.clear();

        for (entity, (transform, text, style)) in
            world.query::<(&Transform, &Text, &TextStyle)>().iter()
        {
            let needs_update = match self.text_cache.get(&entity) {
                Some(cached) => cached.version != text.version,
                None => true,
            };

            if needs_update {
                let render_data = TextRenderData {
                    content: Arc::new(text.content.clone()),
                    position: [transform.position[0], transform.position[1], style.z_index],
                    size: style.size,
                    color: style.color,
                };
                self.text_cache.insert(
                    entity,
                    CachedText {
                        version: text.version,
                        render_data,
                    },
                );
            }

            if let Some(cached) = self.text_cache.get(&entity) {
                self.text_render_buffer.push(cached.render_data.clone());
            }
        }

        self.cache_cleanup_counter += 1;
        if self.cache_cleanup_counter >= 60 {
            self.cache_cleanup_counter = 0;
            let valid_entities: HashSet<Entity> = world
                .query::<&Text>()
                .iter()
                .map(|(entity, _)| entity)
                .collect();
            self.text_cache
                .retain(|entity, _| valid_entities.contains(entity));
        }
    }
}
