use hecs::World;
use pixie::ResourceContainer;

use crate::components::{DNA, PipeTarget, Player, Transform};
use crate::flappy_app::Stage;

/// Process neural network layer
fn process_layer(input_array: Vec<f32>, genes: Vec<&f32>, neuron_count: usize) -> Vec<f32> {
    let mut res = vec![];

    let gene_unit_size = input_array.len() + 1;
    if gene_unit_size * neuron_count != genes.len() {
        panic!("process neural network error!! input and genes count not matched {} , {}, {}", gene_unit_size, neuron_count, genes.len())
    }

    for i in 0..neuron_count {
        let output_value = input_array.iter()
            .zip(
                genes.iter()
                    .skip(gene_unit_size * i)
                    .take(input_array.len()))
            .map(|(i, g)| {
                *i * *g
            })
            .sum::<f32>()
            + genes[gene_unit_size * (i + 1) - 1];

        res.push(output_value);
    }

    res
}

/// Process neural network for AI players
pub fn process_nn(world: &mut World, resources: &mut ResourceContainer) {
    let stage = resources.get::<Stage>().expect("Stage resource not found");

    // Only run when game is in Run stage
    if *stage != Stage::Run {
        return;
    }

    // Find nearest pipe
    let mut pipe_position = [99.0, 0.0];
    for (_entity, (_pipe_target, pipe_tr)) in world.query::<(&PipeTarget, &Transform)>().iter() {
        if pipe_tr.position[0] > -1.5 && pipe_position[0] > pipe_tr.position[0] {
            pipe_position = [pipe_tr.position[0], pipe_tr.position[1]];
        }
    }

    // Process each player's neural network
    for (_entity, (player, p_tr, gene)) in world.query_mut::<(&mut Player, &Transform, &DNA)>() {
        let input_data = vec![
            pipe_position[0] - p_tr.position[0],
            pipe_position[1] - p_tr.position[1],
        ];

        let input_data_size = 2;

        // Layer 1
        let hidden_layer_1_size = gene.hidden_layers[0];
        let layer_1_gene_size = (input_data_size + 1) * hidden_layer_1_size;
        let gene_layer_1 = gene.genes.iter().take(layer_1_gene_size).collect();
        let layer_1 = process_layer(input_data, gene_layer_1, hidden_layer_1_size);

        // Layer 2
        let hidden_layer_2_size = gene.hidden_layers[1];
        let layer_2_gene_size = (hidden_layer_1_size + 1) * hidden_layer_2_size;
        let layer_2_gene = gene.genes.iter().skip(layer_1_gene_size).take(layer_2_gene_size).collect();
        let layer_2 = process_layer(layer_1, layer_2_gene, hidden_layer_2_size);

        // Output layer
        let output_layer_gene_size = hidden_layer_2_size + 1;
        let output_layer_gene = gene.genes.iter().skip(layer_1_gene_size + layer_2_gene_size).take(output_layer_gene_size).collect();
        let output_layer = process_layer(layer_2, output_layer_gene, 1);

        player.jump = output_layer[0] > 0.0f32;
    }
}

#[cfg(test)]
mod tests {
    const EPSILON: f32 = 1e-6;
    use super::*;

    fn approximately_equal(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_process_layer() {
        let input_array = vec![0.2, 0.5];
        let genes = vec![
            0.4, 0.9, 0.01,
            0.3, 0.2, 0.01,
            0.5, 0.1, -0.03,
            0.3, 0.1, 0.07,
            0.9, -0.3, 0.02,
            0.4, 0.9, 0.01,
            0.3, 0.2, 0.01,
            0.5, 0.1, -0.03,
            0.3, 0.1, 0.07,
            0.9, -0.3, 0.02,
            0.4, 0.9, 0.01,
            0.3, 0.2, 0.01,
            0.5, 0.1, -0.03,
            0.3, 0.1, 0.07,
            0.9, -0.3, 0.02
        ];

        let gene = genes.iter().skip(15).take(15).collect();

        let res = process_layer(input_array, gene, 5);
        assert!(approximately_equal(res[0], 0.54f32));
        assert!(approximately_equal(res[1], 0.17f32));
        assert!(approximately_equal(res[2], 0.12f32));
        assert!(approximately_equal(res[3], 0.18f32));
        assert!(approximately_equal(res[4], 0.05f32));
    }
}
