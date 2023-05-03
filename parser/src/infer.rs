use gcd::Gcd;
use std::cmp::min;
use std::collections::{HashSet, VecDeque};
use types::flow::FindAdj;
use types::program::Program;
use types::state::Position;

#[derive(Copy, Clone, Default, Debug)]
pub enum CodelSettings {
    #[default]
    Default,
    Infer,
    Width(u32),
}

pub trait InferCodelWidth: FindAdj {
    fn infer_codel_width(program: &Program) -> u32 {
        let (height, width) = program.dimensions();
        // The idea of this heuristic is that since the codel size is constant across the program,
        // each block must have width and height as a multiple of the codel size.  A somewhat reasonable metric
        // is to explore all colorblocks and compute the gcd of that block's width and height with the program's,
        // and store those minimums, and take the gcds of those gcds.

        let mut gcds = Vec::<u32>::new();
        let mut discovered = HashSet::new();
        let mut queue = VecDeque::from([(0, 0)]);
        let mut next = Vec::<Position>::new();

        while discovered.len() < (height * width) as usize {
            let mut block = HashSet::new();

            let lightness = program.get(*queue.front().unwrap()).unwrap();

            while !queue.is_empty() {
                let pos = queue.pop_front().unwrap();
                block.insert(pos);

                let adjs = <Self as FindAdj>::adjacencies(pos, program, 1);
                let in_block = adjs
                    .iter()
                    .cloned()
                    .filter(|&pos| program.get(pos).unwrap() == lightness)
                    .collect::<HashSet<_>>();

                next.extend(
                    adjs.difference(&in_block)
                        .cloned()
                        .filter(|pos| !discovered.contains(pos)),
                );

                for adj in in_block {
                    if !block.contains(&adj) {
                        queue.push_back(adj);
                    }
                    block.insert(adj);
                }
            }

            let (max_x, _) = *block.iter().max_by_key(|(x, _)| x).unwrap();
            let (min_x, _) = *block.iter().min_by_key(|(x, _)| x).unwrap();
            let (_, max_y) = *block.iter().max_by_key(|(_, y)| y).unwrap();
            let (_, min_y) = *block.iter().min_by_key(|(_, y)| y).unwrap();

            let mut min_width = max_y - min_y + 1; // Initially set to max block width and height
            let mut min_height = max_x - min_x + 1;

            for col in min_y..=max_y {
                for (i, row) in (min_x..=max_x).enumerate() {
                    let p = program.get((row, col));
                    if !p.is_some() || p.unwrap() != lightness {
                        min_width = min(min_width, i as u32);
                        break;
                    }
                }
            }

            for row in min_x..=max_x {
                for (i, col) in (min_y..=max_y).enumerate() {
                    let p = program.get((row, col));
                    if !p.is_some() || p.unwrap() != lightness {
                        min_height = min(min_height, i as u32);
                        break;
                    }
                }
            }

            let gcd_for_block = min(min_width.gcd(height), min_height.gcd(width));

            gcds.push(gcd_for_block);
            queue.extend(&next);
            discovered.extend(block);
            next.clear();
        }

        let res = gcds.into_iter().reduce(|x, y| x.gcd(y)).unwrap();

        if width % res != 0 || height % res != 0 {
            1
        } else {
            res
        }
    }
}
