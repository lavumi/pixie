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

        RenderFrame::new(
            camera_uniform,
            &self.sprite_render_data,
            &self.sprite_atlases,
            &self.text_render_buffer,
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::Camera;

    fn resources_with_camera() -> ResourceContainer {
        let mut resources = ResourceContainer::new();
        resources.insert(Camera::init_orthographic(10.0, 1.0));
        resources
    }

    #[test]
    fn extracts_sprite_and_text_render_data() {
        let mut world = World::new();
        world.spawn((
            Transform::with_rotation([2.0, 3.0, 0.5], [4.0, 5.0], 0.25),
            Sprite {
                atlas: "main".to_string(),
                uv: [0.0, 0.5, 0.5, 1.0],
            },
        ));
        world.spawn((
            Transform::new([6.0, 7.0, 0.0], [1.0, 1.0]),
            Text {
                content: "score".to_string(),
                version: 0,
            },
            TextStyle {
                size: [2.0, 3.0],
                color: [0.1, 0.2, 0.3],
                z_index: 0.75,
            },
        ));

        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();
        let frame = extractor.extract(&world, &resources);
        let batches = frame.sprite_batches().collect::<Vec<_>>();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].0, "main");
        assert_eq!(batches[0].1.len(), 1);
        assert_eq!(batches[0].1[0].position, [2.0, 3.0, 0.5]);
        assert_eq!(batches[0].1[0].size, [4.0, 5.0]);
        assert_eq!(batches[0].1[0].rotation, 0.25);
        assert_eq!(batches[0].1[0].uv, [0.0, 0.5, 0.5, 1.0]);

        assert_eq!(frame.texts().len(), 1);
        assert_eq!(frame.texts()[0].content.as_str(), "score");
        assert_eq!(frame.texts()[0].position, [6.0, 7.0, 0.75]);
        assert_eq!(frame.texts()[0].size, [2.0, 3.0]);
        assert_eq!(frame.texts()[0].color, [0.1, 0.2, 0.3]);
    }

    #[test]
    fn clears_previous_frame_data_on_next_extraction() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::default(),
            Sprite {
                atlas: "main".to_string(),
                uv: [0.0, 1.0, 0.0, 1.0],
            },
        ));
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();

        {
            let frame = extractor.extract(&world, &resources);
            assert_eq!(frame.sprite_batches().count(), 1);
        }

        world.despawn(entity).unwrap();
        let frame = extractor.extract(&world, &resources);

        assert_eq!(frame.sprite_batches().count(), 0);
        assert_eq!(frame.sprite_atlases().count(), 0);
    }

    #[test]
    fn groups_sprites_into_atlas_batches() {
        let mut world = World::new();
        for atlas in ["first", "second", "first"] {
            world.spawn((
                Transform::default(),
                Sprite {
                    atlas: atlas.to_string(),
                    uv: [0.0, 1.0, 0.0, 1.0],
                },
            ));
        }
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();
        let frame = extractor.extract(&world, &resources);
        let batch_sizes = frame
            .sprite_batches()
            .map(|(atlas, sprites)| (atlas, sprites.len()))
            .collect::<HashMap<_, _>>();

        assert_eq!(batch_sizes.len(), 2);
        assert_eq!(batch_sizes["first"], 2);
        assert_eq!(batch_sizes["second"], 1);
    }

    #[test]
    fn empty_world_produces_empty_draw_input() {
        let world = World::new();
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();
        let frame = extractor.extract(&world, &resources);

        assert_eq!(frame.sprite_batches().count(), 0);
        assert_eq!(frame.sprite_atlases().count(), 0);
        assert!(frame.texts().is_empty());
        assert_eq!(
            frame.camera_uniform(),
            resources.get::<Camera>().unwrap().get_view_proj()
        );
    }
}
