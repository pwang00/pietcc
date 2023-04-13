use parser::decode::DecodeInstruction;
use parser::infer::InferCodelWidth;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::rc::Rc;
use types::color::{Lightness, Lightness::*};
use types::flow::{Codel, DirVec, Direction, FindAdj, FURTHEST, MOVE_IN};
use types::program::Program;
use types::state::{Position, ENTRY};

type AdjacencyList = HashMap<Rc<ColorBlock>, HashMap<Rc<ColorBlock>, Vec<DirVec>>>;
pub struct CFGGenerator<'a> {
    program: &'a Program<'a>,
    adjacencies: AdjacencyList,
    codel_size: u32,
}

#[derive(Debug, Eq)]
pub struct ColorBlock {
    label: String,
    lightness: Lightness,
    region: HashSet<Position>,
}

impl ColorBlock {
    pub fn new(label: String, lightness: Lightness, region: HashSet<Position>) -> Self {
        Self {
            label,
            lightness,
            region,
        }
    }

    pub fn contains(&self, pos: Position) -> bool {
        self.region.contains(&pos)
    }

    pub fn get_region(&self) -> &HashSet<Position> {
        &self.region
    }
}

impl Hash for ColorBlock {
    // Can just use the label as hash and this is distinct for each distinct color block
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state)
    }
}

impl PartialEq for ColorBlock {
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }

    // Can simply compare labels
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl<'a> DecodeInstruction for CFGGenerator<'a> {}

impl<'a> FindAdj for CFGGenerator<'a> {}

impl<'a> CFGGenerator<'a> {
    // Returns the list of adjacencies for a given position and whether or not it is a boundary
    pub fn new(prog: &'a Program, codel_size: u32) -> Self {
        CFGGenerator {
            program: prog,
            adjacencies: HashMap::new(),
            codel_size,
        }
    }

    fn possible_exits(&self, cb: &HashSet<Position>) -> Vec<(Position, DirVec)> {
        // first char is dp orientation, second char is cc orientation
        (0..8)
            .map(|x| {
                cb.iter()
                    .max_by_key(FURTHEST[x])
                    .map(|&(x, y)| (x, y, self.codel_size))
                    .map(MOVE_IN[x / 2])
                    .unwrap()
            })
            .zip(
                vec![
                    (Direction::Right, Codel::Left),
                    (Direction::Right, Codel::Right),
                    (Direction::Down, Codel::Left),
                    (Direction::Down, Codel::Right),
                    (Direction::Left, Codel::Left),
                    (Direction::Left, Codel::Right),
                    (Direction::Up, Codel::Left),
                    (Direction::Up, Codel::Right),
                ]
                .into_iter(),
            )
            .filter(|&(pos, _)| {
                let lightness = self.program.get(pos);
                lightness.is_some() && lightness.unwrap() != &Black
            })
            .collect::<Vec<_>>()
    }

    fn explore_region(&self, entry: Position) -> Rc<ColorBlock> {
        // So we don't do duplicate discovery
        if let Some(block) = self.adjacencies.keys().find(|cb| cb.contains(entry)) {
            return block.clone();
        }

        let mut discovered: HashSet<Position> = HashSet::from([entry]);
        let mut queue: VecDeque<Position> = VecDeque::from([entry]);
        let lightness = *self.program.get(entry).unwrap();

        while !queue.is_empty() {
            let pos = queue.pop_front().unwrap();
            let adjs = Self::adjacencies(pos, &self.program, self.codel_size);

            let in_block = adjs
                .iter()
                .filter(|&&pos| *self.program.get(pos).unwrap() == lightness)
                .collect::<Vec<_>>();

            // Adds adjacencies that are in the current color block to queue
            for adj in in_block {
                if !discovered.contains(adj) {
                    queue.push_back(*adj);
                }
                discovered.insert(*adj);
            }
        }

        let (r, c) = discovered.iter().max_by_key(|&(r, c)| (r, c)).unwrap();
        let label = format!("{}_{}_{}", lightness.to_string(), r, c);
        Rc::new(ColorBlock::new(label, lightness, discovered))
    }

    pub fn analyze(&mut self) {
        let init_block = self.explore_region(ENTRY);
        let mut discovered_regions = HashSet::from([init_block.clone()]);
        let mut queue = VecDeque::<Rc<ColorBlock>>::from([init_block]);

        while !queue.is_empty() {
            let curr_block = queue.pop_front().unwrap();
            let curr_exits = self.possible_exits(curr_block.get_region());
            let mut bordering = HashMap::<Rc<ColorBlock>, Vec<DirVec>>::new();

            discovered_regions.insert(curr_block.clone());

            for (boundary, dir) in curr_exits {
                let adj_block = self.explore_region(boundary);
                bordering
                    .entry(adj_block.clone())
                    .and_modify(|vec| vec.push(dir))
                    .or_insert(Vec::new());

                if !discovered_regions.contains(&adj_block) {
                    queue.push_back(adj_block)
                }
            }
            self.adjacencies.insert(curr_block, bordering);
        }
    }

    pub fn get_state(&self) -> &Self {
        &self
    }
}

#[cfg(test)]

mod test {
    use super::*;
    use std::{
        collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
        hash::Hasher,
    };
    use types::color::{Hue::*, Lightness, Lightness::*};

    fn get_hash<T: Hash>(obj: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        obj.hash(&mut hasher);
        hasher.finish()
    }
    #[test]
    fn test_colorblock_eq_hash() {
        let cb1 = ColorBlock {
            label: String::from("LightRed_1_2"),
            lightness: Light(Red),
            region: HashSet::from([
                (2, 2),
                (1, 0),
                (0, 0),
                (0, 2),
                (2, 1),
                (0, 1),
                (1, 2),
                (2, 0),
            ]),
        };
        let cb2 = ColorBlock {
            label: String::from("LightRed_1_2"),
            lightness: Light(Red),
            region: HashSet::from([
                (0, 0),
                (1, 0),
                (0, 1),
                (2, 2),
                (2, 0),
                (1, 2),
                (0, 2),
                (2, 1),
            ]),
        };

        assert_eq!(cb1, cb2);
        assert_eq!(get_hash(&cb1), get_hash(&cb2));
    }
    #[test]
    fn test_program() {
        use crate::cfg::ColorBlock;

        use super::CFGGenerator;

        let vec = vec![
            Light(Red),
            Light(Green),
            Light(Green),
            Light(Red),
            Light(Green),
            Light(Green),
            Light(Red),
            Light(Red),
            Light(Green),
            Light(Red),
            Light(Yellow),
            Light(Yellow),
            Light(Yellow),
            Light(Yellow),
            Light(Yellow),
            Light(Yellow),
            Light(Yellow),
            Light(Yellow),
        ];

        let prog = Program::new(&vec, 6, 3);
        let mut cfg_gen = CFGGenerator::new(&prog, 1);

        cfg_gen.analyze();

        let adjacencies = &cfg_gen.get_state().adjacencies;
        println!("{:?}", adjacencies);
    }
}
