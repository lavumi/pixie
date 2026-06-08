use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hecs::{Entity, World};

use crate::components::{Sprite, Text, TextStyle, Transform};
use crate::renderer::{RenderFrame, SpriteRenderData, TextRenderData};
use crate::resources::ResourceContainer;

struct CachedText {
    version: u64,
    render_data: TextRenderData,
}

#[derive(Default)]
pub struct RenderWorldExtractor {
    sprite_render_data: HashMap<String, Vec<SpriteRenderData>>,
    sprite_atlases: Vec<String>,
    active_sprite_atlases: HashSet<String>,
    text_cache: HashMap<Entity, CachedText>,
    text_render_buffer: Vec<TextRenderData>,
    cache_cleanup_counter: u32,
}

impl RenderWorldExtractor {
    pub fn with_capacity(sprite_atlas_count: usize, text_count: usize) -> Self {
        Self {
            sprite_render_data: HashMap::with_capacity(sprite_atlas_count),
            sprite_atlases: Vec::with_capacity(sprite_atlas_count),
            active_sprite_atlases: HashSet::with_capacity(sprite_atlas_count),
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

        self.extract_sprites(world);
        self.extract_texts(world);

        RenderFrame {
            camera_uniform,
            sprite_render_data: &self.sprite_render_data,
            sprite_atlases: &self.sprite_atlases,
            texts: &self.text_render_buffer,
        }
    }

    fn extract_sprites(&mut self, world: &World) {
        for sprites in self.sprite_render_data.values_mut() {
            sprites.clear();
        }
        self.sprite_atlases.clear();
        self.active_sprite_atlases.clear();

        for (_entity, (transform, sprite)) in world.query::<(&Transform, &Sprite)>().iter() {
            if self.active_sprite_atlases.insert(sprite.atlas.clone()) {
                self.sprite_atlases.push(sprite.atlas.clone());
            }

            self.sprite_render_data
                .entry(sprite.atlas.clone())
                .or_default()
                .push(SpriteRenderData {
                    position: transform.position,
                    size: transform.size,
                    rotation: transform.rotation,
                    uv: sprite.uv,
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
