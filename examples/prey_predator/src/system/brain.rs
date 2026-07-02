use hecs::World;

use crate::components::{AgentMotion, Brain, BrainOutput, Vision, VisionOutput};

pub fn process_brains(world: &mut World) {
    let mut inputs = Vec::new();
    let mut activations = Vec::new();
    let mut next = Vec::new();

    for (_entity, (vision, vision_output, brain, brain_output, motion)) in world
        .query::<(
            &Vision,
            &VisionOutput,
            &Brain,
            &mut BrainOutput,
            &mut AgentMotion,
        )>()
        .iter()
    {
        vision_output.write_inputs(vision, &mut inputs);
        brain
            .genome
            .evaluate_with_buffers(&brain.shape, &inputs, &mut activations, &mut next);
        assert_eq!(
            activations.len(),
            2,
            "prey predator brains must produce rotation and speed outputs"
        );

        brain_output.angular_velocity = activations[0];
        brain_output.speed = activations[1];
        motion.angular_velocity = activations[0] * motion.max_abs_angular_velocity;
        motion.speed = activations[1] * motion.max_abs_speed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Species, VisionHit};
    use crate::neural_network::{Genome, NetworkShape};

    #[test]
    fn brain_outputs_scale_to_agent_motion_limits() {
        let mut world = World::new();
        let vision = Vision::new(10.0, 0.0, 1.0);
        let shape = NetworkShape::new(vision.input_len(), &[], 2);
        let target = world.spawn(());
        let expected_rotation = 0.5_f32.tanh();
        let expected_speed = (-0.25_f32).tanh();
        let genome = Genome::from_genes(
            &shape,
            vec![
                0.0, 0.0, 0.0, 0.5, // rotation bias
                0.0, 0.0, 0.0, -0.25, // speed bias
            ],
        );
        let agent = world.spawn((
            vision,
            VisionOutput {
                hits: vec![Some(VisionHit {
                    target,
                    species: Species::Prey,
                    distance: 2.0,
                })],
            },
            Brain::new(shape, genome),
            BrainOutput::default(),
            AgentMotion::new(0.0, 4.0, 0.0, 2.0),
        ));

        process_brains(&mut world);

        let motion = world.get::<&AgentMotion>(agent).unwrap();
        assert!((motion.angular_velocity - expected_rotation * 2.0).abs() < 1.0e-6);
        assert!((motion.speed - expected_speed * 4.0).abs() < 1.0e-6);
        let output = world.get::<&BrainOutput>(agent).unwrap();
        assert!((output.angular_velocity - expected_rotation).abs() < 1.0e-6);
        assert!((output.speed - expected_speed).abs() < 1.0e-6);
    }
}
