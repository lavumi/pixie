use hecs::World;
use crate::resources::ResourceContainer;

/// Update camera system - currently a no-op placeholder
/// Can be used to implement camera following logic
pub fn update_camera(_world: &mut World, _resources: &mut ResourceContainer) {
    // Placeholder for camera update logic
    // Example: Follow player entity
    // let camera = resources.get_mut::<Camera>().unwrap();
    // if let Some((_, (transform,))) = world.query_mut::<(&Transform,)>()
    //     .with::<Player>()
    //     .into_iter()
    //     .next()
    // {
    //     camera.move_camera([transform.position[0], transform.position[1]]);
    // }
}