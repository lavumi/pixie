use std::collections::HashMap;
use hecs::World;
use winit::event::WindowEvent;
use crate::renderer::{TileRenderData, TextRenderData};
use crate::resources::ResourceContainer;

/// Application trait - defines game-specific logic interface
pub trait Application {
    /// Initialize - register components, insert resources, spawn entities
    fn init(&mut self, world: &mut World, resources: &mut ResourceContainer);

    /// Update every frame (rendering/input handling, non-physics)
    fn update(&mut self, world: &mut World, resources: &mut ResourceContainer, dt: f32);

    /// Fixed timestep update (physics only). Default is no-op
    fn fixed_update(&mut self, _world: &mut World, _resources: &mut ResourceContainer, _fixed_dt: f32) { }

    /// Handle input (returns whether the event was consumed)
    fn handle_input(&mut self, world: &mut World, resources: &mut ResourceContainer, event: &WindowEvent) -> bool;

    /// Provide tile rendering data
    fn get_tile_instances(&self, world: &World, resources: &ResourceContainer) -> HashMap<String, Vec<TileRenderData>>;

    /// Provide text rendering data
    fn get_text_instances(&self, world: &World, resources: &ResourceContainer) -> Vec<TextRenderData>;

    /// Whether to run fixed step (control via pause/state). Default: always run
    fn should_run_fixed(&self, _world: &World, _resources: &ResourceContainer) -> bool { true }
}
