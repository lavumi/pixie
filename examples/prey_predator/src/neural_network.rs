use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkShape {
    layer_sizes: Vec<usize>,
}

impl NetworkShape {
    pub fn new(input_size: usize, hidden_layers: &[usize], output_size: usize) -> Self {
        assert!(input_size > 0, "network input size must be positive");
        assert!(output_size > 0, "network output size must be positive");
        assert!(
            hidden_layers.iter().all(|size| *size > 0),
            "network hidden layer sizes must be positive"
        );

        let mut layer_sizes = Vec::with_capacity(hidden_layers.len() + 2);
        layer_sizes.push(input_size);
        layer_sizes.extend_from_slice(hidden_layers);
        layer_sizes.push(output_size);
        Self { layer_sizes }
    }

    pub fn input_size(&self) -> usize {
        self.layer_sizes[0]
    }

    pub fn output_size(&self) -> usize {
        *self
            .layer_sizes
            .last()
            .expect("network shape always has an output layer")
    }

    pub fn parameter_count(&self) -> usize {
        self.layer_sizes
            .windows(2)
            .map(|layers| (layers[0] + 1) * layers[1])
            .sum()
    }

    fn connections(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.layer_sizes
            .windows(2)
            .map(|layers| (layers[0], layers[1]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Genome {
    genes: Vec<f32>,
}

impl Genome {
    pub fn random<R: Rng + ?Sized>(shape: &NetworkShape, rng: &mut R) -> Self {
        Self::random_with_output_biases(shape, &vec![0.0; shape.output_size()], rng)
    }

    pub fn random_with_output_biases<R: Rng + ?Sized>(
        shape: &NetworkShape,
        output_biases: &[f32],
        rng: &mut R,
    ) -> Self {
        assert_eq!(
            output_biases.len(),
            shape.output_size(),
            "output bias count does not match network shape"
        );

        let mut genes = Vec::with_capacity(shape.parameter_count());
        let connection_count = shape.layer_sizes.len() - 1;
        for (layer_index, (input_size, output_size)) in shape.connections().enumerate() {
            let limit = (6.0 / (input_size + output_size) as f32).sqrt();
            if layer_index + 1 == connection_count {
                for bias in output_biases {
                    for _ in 0..input_size {
                        genes.push(rng.gen_range(-limit..=limit));
                    }
                    genes.push(*bias);
                }
            } else {
                for _ in 0..output_size {
                    for _ in 0..input_size {
                        genes.push(rng.gen_range(-limit..=limit));
                    }
                    genes.push(0.0);
                }
            }
        }
        Self { genes }
    }

    #[cfg(test)]
    pub fn from_genes(shape: &NetworkShape, genes: Vec<f32>) -> Self {
        assert_eq!(
            genes.len(),
            shape.parameter_count(),
            "genome length does not match network shape"
        );
        Self { genes }
    }

    pub fn evaluate(&self, shape: &NetworkShape, inputs: &[f32]) -> Vec<f32> {
        let mut activations = Vec::new();
        let mut next = Vec::new();
        self.evaluate_with_buffers(shape, inputs, &mut activations, &mut next);
        activations
    }

    pub fn evaluate_with_buffers(
        &self,
        shape: &NetworkShape,
        inputs: &[f32],
        activations: &mut Vec<f32>,
        next: &mut Vec<f32>,
    ) {
        assert_eq!(
            self.genes.len(),
            shape.parameter_count(),
            "genome length does not match network shape"
        );
        assert_eq!(
            inputs.len(),
            shape.input_size(),
            "input length does not match network shape"
        );

        activations.clear();
        activations.extend_from_slice(inputs);
        let mut gene_index = 0;
        for (input_size, output_size) in shape.connections() {
            next.clear();
            next.reserve(output_size);
            for _ in 0..output_size {
                let weights = &self.genes[gene_index..gene_index + input_size];
                gene_index += input_size;
                let bias = self.genes[gene_index];
                gene_index += 1;
                let weighted_sum = activations
                    .iter()
                    .zip(weights)
                    .fold(bias, |sum, (input, weight)| sum + input * weight);
                next.push(weighted_sum.tanh());
            }
            std::mem::swap(activations, next);
        }
    }

    pub fn mutated<R: Rng + ?Sized>(&self, config: &MutationConfig, rng: &mut R) -> Self {
        config.validate();
        let mut child = self.clone();
        for gene in &mut child.genes {
            if rng.gen_bool(config.reset_rate as f64) {
                *gene = rng.gen_range(-1.0..=1.0);
            } else if rng.gen_bool(config.mutation_rate as f64) {
                *gene += standard_normal(rng) * config.mutation_sigma;
            }
            *gene = gene.clamp(-config.max_abs_gene, config.max_abs_gene);
        }
        child
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MutationConfig {
    pub mutation_rate: f32,
    pub mutation_sigma: f32,
    pub reset_rate: f32,
    pub max_abs_gene: f32,
}

impl MutationConfig {
    fn validate(&self) {
        assert!(
            (0.0..=1.0).contains(&self.mutation_rate),
            "mutation rate must be between zero and one"
        );
        assert!(
            self.mutation_sigma.is_finite() && self.mutation_sigma >= 0.0,
            "mutation sigma must be finite and non-negative"
        );
        assert!(
            (0.0..=1.0).contains(&self.reset_rate),
            "reset rate must be between zero and one"
        );
        assert!(
            self.max_abs_gene.is_finite() && self.max_abs_gene > 0.0,
            "maximum absolute gene value must be finite and positive"
        );
    }
}

fn standard_normal<R: Rng + ?Sized>(rng: &mut R) -> f32 {
    let u1 = rng.gen_range(f32::MIN_POSITIVE..1.0);
    let u2 = rng.gen_range(0.0..1.0);
    (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn parameter_count_includes_one_bias_per_neuron() {
        let shape = NetworkShape::new(27, &[16], 2);

        assert_eq!(shape.parameter_count(), 482);
    }

    #[test]
    fn evaluates_each_layer_with_tanh_activation() {
        let shape = NetworkShape::new(2, &[], 1);
        let genome = Genome::from_genes(&shape, vec![0.5, -1.0, 0.25]);

        let output = genome.evaluate(&shape, &[2.0, 0.5]);

        assert!((output[0] - 0.75_f32.tanh()).abs() < 1.0e-6);
    }

    #[test]
    fn random_genome_matches_shape_and_produces_finite_outputs() {
        let shape = NetworkShape::new(3, &[4], 2);
        let mut rng = StdRng::seed_from_u64(7);
        let genome = Genome::random(&shape, &mut rng);

        let output = genome.evaluate(&shape, &[0.2, 0.4, 0.8]);

        assert_eq!(genome.genes.len(), shape.parameter_count());
        assert_eq!(output.len(), shape.output_size());
        assert!(output.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn output_biases_control_zero_input_defaults() {
        let shape = NetworkShape::new(3, &[], 2);
        let mut rng = StdRng::seed_from_u64(9);
        let genome = Genome::random_with_output_biases(&shape, &[0.0, 1.0], &mut rng);

        let output = genome.evaluate(&shape, &[0.0, 0.0, 0.0]);

        assert_eq!(output[0], 0.0);
        assert!((output[1] - 1.0_f32.tanh()).abs() < 1.0e-6);
    }

    #[test]
    fn initial_population_favors_forward_motion() {
        let shape = NetworkShape::new(27, &[16], 2);
        let inputs: Vec<f32> = [1.0, 0.0, 0.0].repeat(9);
        let mut rng = StdRng::seed_from_u64(13);
        let forward_count = (0..1_000)
            .filter(|_| {
                let genome = Genome::random_with_output_biases(&shape, &[0.0, 1.0], &mut rng);
                genome.evaluate(&shape, &inputs)[1] > 0.0
            })
            .count();

        assert!(forward_count >= 750);
    }

    #[test]
    fn zero_mutation_rates_preserve_parent_genome() {
        let shape = NetworkShape::new(2, &[3], 2);
        let mut rng = StdRng::seed_from_u64(11);
        let parent = Genome::random(&shape, &mut rng);
        let config = MutationConfig {
            mutation_rate: 0.0,
            mutation_sigma: 0.1,
            reset_rate: 0.0,
            max_abs_gene: 5.0,
        };

        let child = parent.mutated(&config, &mut rng);

        assert_eq!(child, parent);
    }
}
