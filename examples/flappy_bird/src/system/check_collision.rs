use specs::{Entities, Join, Read, ReadStorage, System, Write};

use crate::components::{DNA, Pipe, Player, Transform};
use crate::resources::{GeneHandler, Score};

pub struct CheckCollision;





impl<'a> System<'a> for CheckCollision {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Pipe>,
        ReadStorage<'a, Transform>,

        //todo 이걸 여기서 해야 할까?
        ReadStorage<'a, DNA>,
        Write<'a, GeneHandler>,
        Read<'a, Score>
    );

    fn run(&mut self, (entities,players, pipes, transforms, dna,mut gene_handler, score): Self::SystemData) {

        for ( e, _, player_tr, d) in  (&entities, &players, &transforms, &dna).join() {
            let pt =player_tr.position;
            if pt[1] < -7.0  || pt[1] > 9.0{
                gene_handler.set_score(d.index , score.0);
                entities.delete(e).expect("delete player fail!!!");
            }
            for (_, pipe_tr) in  ( & pipes, &transforms).join() {
                let obstacle_point = [
                    if pt[0] > pipe_tr.position[0] + pipe_tr.size[0] * 0.5 {
                        pipe_tr.position[0] + pipe_tr.size[0] * 0.5
                    }
                    else if pt[0] < pipe_tr.position[0] - pipe_tr.size[0] * 0.5 {
                        pipe_tr.position[0] - pipe_tr.size[0] * 0.5
                    }
                    else {
                        pt[0]
                    },
                    if pt[1] > pipe_tr.position[1] + pipe_tr.size[1] * 0.5 {
                        pipe_tr.position[1] + pipe_tr.size[1] * 0.5
                    }
                    else if pt[1] < pipe_tr.position[1] - pipe_tr.size[1] * 0.5 {
                        pipe_tr.position[1] - pipe_tr.size[1] * 0.5
                    }
                    else {
                        pt[1]
                    }
                ];

                let dist_pow = (obstacle_point[0] - pt[0]) * (obstacle_point[0] - pt[0]) + (obstacle_point[1] - pt[1]) * (obstacle_point[1] - pt[1]);
                if dist_pow < 0.2 {
                    gene_handler.set_score(d.index , score.0);
                    entities.delete(e).expect("delete player fail!!!");
                    continue;
                }

            }
        }
    }
}