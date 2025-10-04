use specs::*;
use specs_derive::Component;
use crate::game_configs::GENE_SIZE;

// Re-export generic components from engine
pub use engine::components::*;

#[derive(Component, Clone)]
pub struct Background {
    pub reposition_size : f32,
}

#[derive(Component, Clone)]
pub struct Pipe {
    pub reposition_size : f32,
    pub pipe_index : u8,
}

#[derive(Component, Clone)]
pub struct PipeTarget {}

#[derive(Component, Clone, Default)]
pub struct Player {
    pub force: f32,
    pub jump : bool,
}


//region [ Neural Network ]
#[derive(Component, Clone, Default)]
pub struct NeuralLayer {
    pub weights: Vec<Vec<f32>>,
    pub values : Vec<f32>,
    pub bias : Vec<f32>
}


#[derive(Component, Clone)]
pub struct DNA {
    pub hidden_layers:[usize;2],
    pub genes:[f32;GENE_SIZE],
    pub index:usize,
}


//endregion