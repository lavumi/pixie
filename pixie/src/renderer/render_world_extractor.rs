use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hecs::World;

use crate::components::{Sprite, Text, TextCoordinateSpace, TextStyle, Transform};
use crate::renderer::{RenderFrame, SpriteRenderData, TextRenderData};
use crate::resources::ResourceContainer;
use crate::{AtlasError, AtlasId, DebugDraw, DebugLine, TextureAtlasRegistry};

#[derive(Default)]
pub struct RenderWorldExtractor {
    sprite_render_data: HashMap<AtlasId, Vec<SpriteRenderData>>,
    sprite_atlases: Vec<AtlasId>,
    active_sprite_atlases: HashSet<AtlasId>,
    world_text_render_buffer: Vec<TextRenderData>,
    screen_text_render_buffer: Vec<TextRenderData>,
    debug_lines: Vec<DebugLine>,
}

impl RenderWorldExtractor {
    pub fn with_capacity(sprite_atlas_count: usize, text_count: usize) -> Self {
        Self {
            sprite_render_data: HashMap::with_capacity(sprite_atlas_count),
            sprite_atlases: Vec::with_capacity(sprite_atlas_count),
            active_sprite_atlases: HashSet::with_capacity(sprite_atlas_count),
            world_text_render_buffer: Vec::with_capacity(text_count),
            screen_text_render_buffer: Vec::with_capacity(text_count),
            debug_lines: Vec::with_capacity(256),
        }
    }

    pub fn extract<'a>(
        &'a mut self,
        world: &World,
        resources: &ResourceContainer,
    ) -> Result<RenderFrame<'a>, AtlasError> {
        let camera_uniform = resources
            .get::<crate::resources::Camera>()
            .expect("Camera resource not found")
            .get_view_proj();

        self.extract_sprites(world, resources)?;
        self.extract_texts(world);
        self.extract_debug_lines(resources);

        Ok(RenderFrame::new(
            camera_uniform,
            &self.sprite_render_data,
            &self.sprite_atlases,
            &self.world_text_render_buffer,
            &self.screen_text_render_buffer,
            &self.debug_lines,
        ))
    }

    fn extract_sprites(
        &mut self,
        world: &World,
        resources: &ResourceContainer,
    ) -> Result<(), AtlasError> {
        let registry = resources
            .get::<TextureAtlasRegistry>()
            .expect("TextureAtlasRegistry resource not found");

        for sprites in self.sprite_render_data.values_mut() {
            sprites.clear();
        }
        self.sprite_atlases.clear();
        self.active_sprite_atlases.clear();

        for (entity, (transform, sprite)) in world.query::<(&Transform, &Sprite)>().iter() {
            if !registry.is_loaded(&sprite.atlas) {
                return Err(AtlasError::MissingAtlas {
                    atlas: sprite.atlas.clone(),
                    entity,
                });
            }

            if !self.active_sprite_atlases.contains(&sprite.atlas) {
                let atlas = sprite.atlas.clone();
                self.active_sprite_atlases.insert(atlas.clone());
                self.sprite_atlases.push(atlas.clone());
                self.sprite_render_data.entry(atlas).or_default();
            }

            self.sprite_render_data
                .get_mut(&sprite.atlas)
                .expect("active sprite atlas must have a render batch")
                .push(SpriteRenderData {
                    position: transform.position,
                    size: transform.size,
                    rotation: transform.rotation,
                    uv: sprite.uv,
                });
        }

        Ok(())
    }

    fn extract_texts(&mut self, world: &World) {
        self.world_text_render_buffer.clear();
        self.screen_text_render_buffer.clear();

        for (_, (transform, text, style)) in world.query::<(&Transform, &Text, &TextStyle)>().iter()
        {
            let buffer = match style.coordinate_space {
                TextCoordinateSpace::World => &mut self.world_text_render_buffer,
                TextCoordinateSpace::Screen => &mut self.screen_text_render_buffer,
            };

            buffer.push(TextRenderData {
                content: Arc::new(text.content.clone()),
                position: [transform.position[0], transform.position[1], style.z_index],
                size: style.size,
                color: style.color,
            });
        }
    }

    fn extract_debug_lines(&mut self, resources: &ResourceContainer) {
        self.debug_lines.clear();
        if let Some(debug_draw) = resources.get::<DebugDraw>() {
            self.debug_lines.extend_from_slice(debug_draw.lines());
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
        let mut registry = TextureAtlasRegistry::default();
        for atlas in ["main", "first", "second"] {
            registry.mark_loaded(AtlasId::from(atlas));
        }
        resources.insert(registry);
        resources
    }

    #[test]
    fn extracts_sprite_and_text_render_data() {
        let mut world = World::new();
        world.spawn((
            Transform::with_rotation([2.0, 3.0, 0.5], [4.0, 5.0], 0.25),
            Sprite {
                atlas: "main".into(),
                uv: [0.0, 0.5, 0.5, 1.0],
            },
        ));
        world.spawn((
            Transform::new([6.0, 7.0, 0.0], [1.0, 1.0]),
            Text {
                content: "score".to_string(),
            },
            TextStyle {
                size: [2.0, 3.0],
                color: [0.1, 0.2, 0.3],
                z_index: 0.75,
                ..TextStyle::default()
            },
        ));

        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();
        let frame = extractor.extract(&world, &resources).unwrap();
        let batches = frame.sprite_batches().collect::<Vec<_>>();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].0.as_str(), "main");
        assert_eq!(batches[0].1.len(), 1);
        assert_eq!(batches[0].1[0].position, [2.0, 3.0, 0.5]);
        assert_eq!(batches[0].1[0].size, [4.0, 5.0]);
        assert_eq!(batches[0].1[0].rotation, 0.25);
        assert_eq!(batches[0].1[0].uv, [0.0, 0.5, 0.5, 1.0]);

        assert_eq!(frame.world_texts().len(), 1);
        assert_eq!(frame.world_texts()[0].content.as_str(), "score");
        assert_eq!(frame.world_texts()[0].position, [6.0, 7.0, 0.75]);
        assert_eq!(frame.world_texts()[0].size, [2.0, 3.0]);
        assert_eq!(frame.world_texts()[0].color, [0.1, 0.2, 0.3]);
    }

    #[test]
    fn clears_previous_frame_data_on_next_extraction() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::default(),
            Sprite {
                atlas: "main".into(),
                uv: [0.0, 1.0, 0.0, 1.0],
            },
        ));
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();

        {
            let frame = extractor.extract(&world, &resources).unwrap();
            assert_eq!(frame.sprite_batches().count(), 1);
        }
        let capacity_before = extractor
            .sprite_render_data
            .get(&AtlasId::from("main"))
            .unwrap()
            .capacity();

        world.despawn(entity).unwrap();
        let frame = extractor.extract(&world, &resources).unwrap();

        assert_eq!(frame.sprite_batches().count(), 0);
        assert_eq!(frame.sprite_atlases().count(), 0);
        assert_eq!(
            extractor
                .sprite_render_data
                .get(&AtlasId::from("main"))
                .unwrap()
                .capacity(),
            capacity_before
        );
    }

    #[test]
    fn reflects_text_transform_and_style_changes_on_next_extraction() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::new([1.0, 2.0, 0.0], [1.0, 1.0]),
            Text {
                content: "before".to_string(),
            },
            TextStyle {
                size: [1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                z_index: 0.5,
                ..TextStyle::default()
            },
        ));
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();

        extractor.extract(&world, &resources).unwrap();

        world.get::<&mut Text>(entity).unwrap().content = "after".to_string();
        world.get::<&mut Transform>(entity).unwrap().position = [3.0, 4.0, 0.0];
        {
            let mut style = world.get::<&mut TextStyle>(entity).unwrap();
            style.size = [2.0, 3.0];
            style.color = [0.1, 0.2, 0.3];
            style.z_index = 0.75;
        }

        let frame = extractor.extract(&world, &resources).unwrap();
        let text = &frame.world_texts()[0];

        assert_eq!(text.content.as_str(), "after");
        assert_eq!(text.position, [3.0, 4.0, 0.75]);
        assert_eq!(text.size, [2.0, 3.0]);
        assert_eq!(text.color, [0.1, 0.2, 0.3]);
    }

    #[test]
    fn groups_sprites_into_atlas_batches() {
        let mut world = World::new();
        for atlas in ["first", "second", "first"] {
            world.spawn((
                Transform::default(),
                Sprite {
                    atlas: atlas.into(),
                    uv: [0.0, 1.0, 0.0, 1.0],
                },
            ));
        }
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();
        let frame = extractor.extract(&world, &resources).unwrap();
        let batch_sizes = frame
            .sprite_batches()
            .map(|(atlas, sprites)| (atlas.as_str(), sprites.len()))
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
        let frame = extractor.extract(&world, &resources).unwrap();

        assert_eq!(frame.sprite_batches().count(), 0);
        assert_eq!(frame.sprite_atlases().count(), 0);
        assert!(frame.world_texts().is_empty());
        assert!(frame.screen_texts().is_empty());
        assert!(frame.debug_lines().is_empty());
        assert_eq!(
            frame.camera_uniform(),
            resources.get::<Camera>().unwrap().get_view_proj()
        );
    }

    #[test]
    fn extracts_debug_lines_without_retaining_cleared_submissions() {
        let world = World::new();
        let mut resources = resources_with_camera();
        let mut debug_draw = DebugDraw::default();
        debug_draw.line([1.0, 2.0, 0.5], [3.0, 4.0, 0.5], [0.1, 0.2, 0.3, 0.4], 0.25);
        resources.insert(debug_draw);
        let mut extractor = RenderWorldExtractor::default();

        {
            let frame = extractor.extract(&world, &resources).unwrap();
            assert_eq!(
                frame.debug_lines(),
                &[DebugLine::new(
                    [1.0, 2.0, 0.5],
                    [3.0, 4.0, 0.5],
                    [0.1, 0.2, 0.3, 0.4],
                    0.25,
                )]
            );
        }

        resources.get_mut::<DebugDraw>().unwrap().clear();
        let frame = extractor.extract(&world, &resources).unwrap();
        assert!(frame.debug_lines().is_empty());
    }

    #[test]
    fn separates_screen_text_from_world_text() {
        let mut world = World::new();
        world.spawn((
            Transform::new([10.0, 20.0, 0.0], [1.0, 1.0]),
            Text {
                content: "hud".to_string(),
            },
            TextStyle {
                size: [16.0, 16.0],
                color: [1.0, 1.0, 1.0],
                z_index: 0.5,
                coordinate_space: TextCoordinateSpace::Screen,
            },
        ));
        world.spawn((
            Transform::new([3.0, 4.0, 0.0], [1.0, 1.0]),
            Text {
                content: "label".to_string(),
            },
            TextStyle {
                size: [1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                z_index: 0.5,
                ..TextStyle::default()
            },
        ));

        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();
        let frame = extractor.extract(&world, &resources).unwrap();

        assert_eq!(frame.world_texts().len(), 1);
        assert_eq!(frame.world_texts()[0].content.as_str(), "label");
        assert_eq!(frame.screen_texts().len(), 1);
        assert_eq!(frame.screen_texts()[0].content.as_str(), "hud");
    }

    #[test]
    fn missing_atlas_reports_entity_and_name() {
        let mut world = World::new();
        let entity = world.spawn((
            Transform::default(),
            Sprite {
                atlas: "missing".into(),
                uv: [0.0, 1.0, 0.0, 1.0],
            },
        ));
        let resources = resources_with_camera();
        let mut extractor = RenderWorldExtractor::default();

        let error = match extractor.extract(&world, &resources) {
            Ok(_) => panic!("missing atlas should fail extraction"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            AtlasError::MissingAtlas {
                atlas,
                entity: error_entity,
            } if atlas.as_str() == "missing" && error_entity == entity
        ));
    }
}
