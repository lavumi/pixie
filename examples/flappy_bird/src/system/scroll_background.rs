use hecs::World;
use pixie::ResourceContainer;

use crate::components::{Background, Transform};
use crate::game_configs::GAME_SPEED;
use pixie::DeltaTime;

/// Scroll background elements
pub fn scroll_background(world: &mut World, resources: &mut ResourceContainer) {
    let dt = resources.get::<DeltaTime>().expect("DeltaTime resource not found");

    for (_entity, (scroll, transform)) in world.query_mut::<(&Background, &mut Transform)>() {
        transform.position[0] -= dt.0 * GAME_SPEED;
        if transform.position[0] + transform.size[0] / 2.0 < -6.0 {
            transform.position[0] += scroll.reposition_size;
        }
    }
}
