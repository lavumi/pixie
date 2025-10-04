use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use crate::components::DNA;
use crate::game_configs::GENE_SIZE;



const EVOLVE_OFFSET : f32 = 0.1;
const SURVIVE_RATIO : f32 = 0.1;
pub struct GeneHandler {
    gene_container: Vec<[f32;GENE_SIZE]>,
    fitness: Vec<f32>,
    pub generation: usize,
    rng : ThreadRng,
    gene_count : usize
}

impl Default for GeneHandler {
    fn default() -> Self {
        let mut gene_handler = GeneHandler{
            gene_container : vec![],
            fitness: vec![],
            generation : 0,
            rng : thread_rng(),
            gene_count : 100
        };

        gene_handler.initialize();
        return gene_handler;
    }
}

impl GeneHandler {

    pub fn get_alive_gene(&self , index : usize)-> [f32;GENE_SIZE]{
        return self.gene_container[index].clone()
    }
    pub fn initialize(&mut self){
        for _ in 0..self.gene_count {
            let mut genes = [0f32;GENE_SIZE];
            for i in 0..GENE_SIZE {
                genes[i] = self.rng.gen_range(-16.0..16.0);
            }
            self.gene_container.push(genes);
            self.fitness.push(-1.0f32);
        }
    }
    pub fn get_dna(&mut self) -> DNA{
        let mut index = 0;
        if let Some((found_index, _)) = self.fitness.iter().enumerate().find(
            |(_, &value)| value == -1.0) {
                index = found_index;
                self.fitness[index] = 0.0;
            }


        let genes = self.gene_container[index];
        DNA{
            hidden_layers: [6,4],
            genes,
            index,
        }
    }


    pub fn set_score(&mut self, index: usize, score:f32){
        self.fitness[index] = score;

    }


    fn pick_gene_by_fitness(&mut self, accumulated_array : &Vec<f32>)-> usize{
        let max = accumulated_array.last().unwrap();
        let rnd = self.rng.gen_range(0.0..*max);
        let mut index = 0;
        for i in 0..accumulated_array.len() {
            if accumulated_array[i] > rnd {
                index = i;
                break;
            }
        }

        return index;
    }

    pub fn process_generation(&mut self){

        let mut next_generation_genes = vec![];


        //1. 상위 10%는 그대로 이어감
        let mut indexed_fitness: Vec<(usize, f32)> = self.fitness.iter().cloned().enumerate().collect();
        indexed_fitness.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let sorted_indices: Vec<usize> = indexed_fitness.iter().map(|(idx, _)| *idx).collect();
        let survive_count = (self.gene_count as f32 * SURVIVE_RATIO) as usize;
        for i in 0..survive_count {
            next_generation_genes.push( self.gene_container[sorted_indices[i]].clone());
        }

        let mut accumulated_fitness = vec![0.0f32; self.fitness.len()];
        let mut sum = 0.0f32;
        for (i, &element) in self.fitness.iter().enumerate() {
            sum += element;
            accumulated_fitness[i] = sum;
        }


        let average = self.fitness.iter().sum::<f32>() / self.fitness.len() as f32;
        let probability = 1.0f32 / f32::max(1.0, average);


        let remain_gene_count = self.gene_count - survive_count;
        for _ in 0..remain_gene_count {
            let gene_index_0 = self.pick_gene_by_fitness(&accumulated_fitness);
            let gene_index_1 = self.pick_gene_by_fitness(&accumulated_fitness);


            let new_gene = if gene_index_0 == gene_index_1 {
                self.evolve( gene_index_0 , probability)
            }
            else {
                self.make_child(gene_index_0,gene_index_1, probability)
            };

            next_generation_genes.push(new_gene);
        }


        for score in self.fitness.iter_mut() {
            *score = -1.0f32;
        }
        self.generation += 1;

        assert_eq!(self.gene_container.len() , next_generation_genes.len());
        self.gene_container = next_generation_genes;
    }

    // fn make_genes(&mut self) -> [f32;GENE_SIZE]{
    //     let mut genes = [0f32;GENE_SIZE];
    //     for i in 0..GENE_SIZE {
    //         genes[i] = self.rng.gen_range(-16.0..16.0);
    //     }
    //     return genes;
    // }

    fn evolve(&mut self, gene_index: usize, probability : f32 )-> [f32;GENE_SIZE]{
        let mut gene = self.gene_container[gene_index].clone();
        // let mut evolve_count = 0;
        for g in gene.iter_mut() {
            let change : f32 = self.rng.gen_range(0.0..1.0);
            if change < probability {
                let offset = self.rng.gen_range(-EVOLVE_OFFSET..EVOLVE_OFFSET);
                *g = *g + offset;
                // evolve_count+=1;
            }
        }

        return gene;
        // log::info!("{:.5} : evolve {}" , probability, evolve_count);
    }

    fn make_child(&mut self, gene_index_0: usize,gene_index_1: usize, probability : f32 )-> [f32;GENE_SIZE]{
        let mut gene = [0.0f32;GENE_SIZE];
        let cross_point = self.rng.gen_range(0..GENE_SIZE);
        for i in 0..GENE_SIZE {
            let change : f32 = self.rng.gen_range(0.0..1.0);
            if change < probability {
                gene[i] = self.rng.gen_range(-16.0..16.0);
            }
            else {
                let distrib =1.0f32 +  self.rng.gen_range(0.0f32..probability);
                gene[i] = if i < cross_point {
                    self.gene_container[gene_index_0][i]
                } else {
                    self.gene_container[gene_index_1][i]
                } * distrib;
            }
        }


        return gene;
    }
}

#[cfg(test)]
mod test{
    use super::*;


    #[test]
    fn test_evolve(){
        let mut gene_handler = GeneHandler::default();

        for i in 0..gene_handler.gene_count {
            gene_handler.set_score( i , i as f32 * 0.2 );
        }

        let average = gene_handler.fitness.iter().sum::<f32>() / gene_handler.fitness.len() as f32;
        let probability = 1.0f32 / f32::max(1.0, average);
        // log::info!("before : \t{:?}" ,gene_handler.gene_container[0] );
        gene_handler.evolve(0 , probability);
        // log::info!("after : \t{:?}" ,gene_handler.gene_container[0] );
    }
}