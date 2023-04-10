use gcd::Gcd;
use rand::Rng;
use std::collections::{HashSet, VecDeque};
use types::flow::FindAdj;
use types::program::Program;

pub trait InferCodelWidth: FindAdj {
    fn infer_codel_width(program: &Program) -> u32 {
        let (height, width) = program.dimensions();
        // The idea of this heuristic is that since the codel size is constant across the program,
        // each block must have width and height as a multiple of the codel size.  A somewhat reasonable metric
        // is to explore a few arbitrarily chosen blocks and compute the gcd of that block's width and height with the program's,
        // and store those minimums, and fold those gcds.

        let mut gcds = Vec::<u32>::new();

        for _ in 0..4 {
            let ex = rand::thread_rng().gen_range(0..height);
            let ey = rand::thread_rng().gen_range(0..width);

            let mut queue = VecDeque::from([(ex, ey)]);
            let mut discovered = HashSet::new();
            let lightness = program.get((ex, ey)).unwrap();

            while !queue.is_empty() {
                let pos = queue.pop_front().unwrap();
                discovered.insert(pos);

                let adjs = <Self as FindAdj>::adjacencies(pos, program, 1);
                let in_block = adjs
                    .iter()
                    .filter(|&&pos| program.get(pos).unwrap() == lightness)
                    .collect::<Vec<_>>();

                for adj in in_block {
                    if !discovered.contains(adj) {
                        queue.push_back(*adj);
                    }
                    discovered.insert(*adj);
                }
            }

            let (max_x, _) = discovered.iter().max_by_key(|(x, _)| x).unwrap();
            let (min_x, _) = discovered.iter().min_by_key(|(x, _)| x).unwrap();
            let (_, max_y) = discovered.iter().max_by_key(|(_, y)| y).unwrap();
            let (_, min_y) = discovered.iter().min_by_key(|(_, y)| y).unwrap();

            gcds.push(std::cmp::min(
                (max_x - min_x + 1).gcd(height),
                (max_y - min_y + 1).gcd(width),
            ))
        }
        gcds.into_iter().reduce(|x, y| x.gcd(y)).unwrap()
    }
}
