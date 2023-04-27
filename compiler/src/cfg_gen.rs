use parser::decode::DecodeInstruction;
use parser::infer::{CodelSettings, InferCodelWidth};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;
use types::color::{Lightness, Lightness::*};
use types::flow::{DirVec, FindAdj, FURTHEST, MOVE_IN};
use types::instruction::Instruction;
use types::program::Program;
use types::state::{Position, ENTRY};

use crate::consts::{DIRECTIONS, ROTATE_ORDERING};

pub(crate) type Node = Rc<ColorBlock>;
pub(crate) type Info = Vec<(DirVec, DirVec, Option<Instruction>)>;
pub(crate) type Adjacencies = HashMap<Node, Info>;
pub(crate) type CFG = HashMap<Node, Adjacencies>;

#[allow(unused)]
pub struct CFGGenerator<'a> {
    program: &'a Program<'a>,
    adjacencies: CFG,
    codel_width: u32,
}

#[allow(unused)]
#[derive(Eq)]
pub(crate) struct ColorBlock {
    label: String,
    lightness: Lightness,
    region: HashSet<Position>,
}

#[allow(unused)]
impl ColorBlock {
    pub(crate) fn new(label: String, lightness: Lightness, region: HashSet<Position>) -> Self {
        Self {
            label,
            lightness,
            region,
        }
    }

    pub(crate) fn contains(&self, pos: Position) -> bool {
        self.region.contains(&pos)
    }

    pub(crate) fn get_region(&self) -> &HashSet<Position> {
        &self.region
    }

    pub(crate) fn get_label(&self) -> &String {
        &self.label
    }

    pub(crate) fn get_region_size(&self) -> u64 {
        self.region.len() as u64
    }

    pub(crate) fn get_lightness(&self) -> Lightness {
        self.lightness
    }
}

impl fmt::Debug for ColorBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColorBlock")
            .field("label", &self.label)
            .field("size", &self.region.len())
            .finish()
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

impl<'a> InferCodelWidth for CFGGenerator<'a> {}

#[allow(unused)]
impl<'a> CFGGenerator<'a> {
    // Returns the list of adjacencies for a given position and whether or not it is a boundary
    pub fn new(prog: &'a Program, codel_settings: CodelSettings) -> Self {
        let codel_width = match codel_settings {
            CodelSettings::Default => 1,
            CodelSettings::Infer => Self::infer_codel_width(prog),
            CodelSettings::Width(codel_width) => codel_width,
        };

        CFGGenerator {
            program: prog,
            adjacencies: HashMap::new(),
            codel_width,
        }
    }

    fn possible_exits(&self, cb: &HashSet<Position>) -> Vec<(Position, DirVec)> {
        // first char is dp orientation, second char is cc orientation
        (0..8)
            .map(|x| {
                cb.iter()
                    .max_by_key(FURTHEST[x])
                    .map(|&(x, y)| (x, y, self.codel_width))
                    .map(MOVE_IN[x / 2])
                    .unwrap()
            })
            .zip(DIRECTIONS.into_iter())
            .filter(|&(pos, _)| {
                let lightness = self.program.get(pos);
                lightness.is_some() && lightness.unwrap() != &Black
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn trace_white(&self, entry: Position, dir: DirVec) -> Option<(Position, DirVec)> {
        let (mut x, mut y) = entry;
        let (mut dp, mut cc) = dir;
        let mut retries = 0;

        while retries < 8 {
            let next_pos = Some((x, y, self.codel_width))
                .map(MOVE_IN[dp as usize])
                .unwrap();

            let lightness = self.program.get(next_pos);

            if lightness.is_none() || lightness == Some(&Black) {
                dp = dp.rotate(1);
                cc = cc.switch(1);
                retries += 1;
                continue;
            }

            if lightness != Some(&White) {
                return Some((next_pos, (dp, cc)));
            }

            (x, y) = next_pos
        }
        return None;
    }

    fn explore_region(&self, entry: Position) -> Node {
        // So we don't do duplicate discovery
        if let Some(block) = self.adjacencies.keys().find(|cb| cb.contains(entry)) {
            return block.clone();
        }

        let mut discovered: HashSet<Position> = HashSet::from([entry]);
        let mut queue: VecDeque<Position> = VecDeque::from([entry]);
        let lightness = *self.program.get(entry).unwrap();

        while !queue.is_empty() {
            let pos = queue.pop_front().unwrap();
            let adjs = Self::adjacencies(pos, &self.program, self.codel_width);

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

        let (r, c) = discovered.iter().min_by_key(|&(r, c)| (r, c)).unwrap();
        let label = if discovered.get(&(0, 0)).is_some() {
            format!("Entry")
        } else {
            format!("{}_{}_{}", lightness.to_string(), r, c)
        };
        Rc::new(ColorBlock::new(label, lightness, discovered))
    }

    pub(crate) fn analyze(&mut self) {
        let init_block = self.explore_region(ENTRY);
        let mut discovered_regions = HashSet::from([init_block.clone()]);
        let mut queue = VecDeque::<Rc<ColorBlock>>::from([init_block]);

        while !queue.is_empty() {
            let curr_block = queue.pop_front().unwrap();
            let curr_exits = self.possible_exits(curr_block.get_region());
            let mut bordering = Adjacencies::new();

            discovered_regions.insert(curr_block.clone());

            for (boundary, dir) in curr_exits {
                let adj_block = self.explore_region(boundary);
                let instr =
                    Self::decode_instr(curr_block.get_lightness(), adj_block.get_lightness());
                    
                if adj_block.get_lightness() != White {
                    bordering
                        .entry(adj_block.clone())
                        .and_modify(|vec| vec.push((dir, dir, instr)))
                        .or_insert(Vec::from([(dir, dir, instr)]));
                } else {
                    let exit_state = self.trace_white(boundary, dir);
                    if let Some((next_pos, next_dir)) = exit_state {
                        let white_adj_lightness =
                            self.program.get(next_pos).map(|lightness| *lightness);

                        let new_adj_block = self.explore_region(next_pos);

                        bordering
                            .entry(new_adj_block.clone())
                            .and_modify(|vec| vec.push((dir, next_dir, None)))
                            .or_insert(Vec::from([(dir, next_dir, None)]));

                        if !discovered_regions.contains(&new_adj_block) {
                            discovered_regions.insert(new_adj_block.clone());
                            queue.push_back(new_adj_block)
                        }
                    }
                }

                if !discovered_regions.contains(&adj_block) {
                    discovered_regions.insert(adj_block.clone());
                    queue.push_back(adj_block)
                }
            }
            if curr_block.get_lightness() != White {
                self.adjacencies.insert(curr_block, bordering);
            }
        }
    }

    pub(crate) fn get_state(&self) -> &Self {
        &self
    }

    pub(crate) fn get_cfg(&self) -> &CFG {
        &self.adjacencies
    }
}

#[cfg(test)]

mod test {
    use super::*;
    use parser::loader::Loader;
    use std::{
        collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
        fs,
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
        let prog = Loader::convert("../images/hw1-1.png").unwrap();
        let mut cfg_gen = CFGGenerator::new(&prog, 1);

        println!("loaded");
        cfg_gen.analyze();

        let adjacencies = &cfg_gen.get_state().adjacencies;

        for node in adjacencies.keys() {
            if node.get_label() == "RegYellow_2_3" {
                println!("{:?}", adjacencies.get(node))
            }
        }
    }
}
