use crate::decode::DecodeInstruction;
use crate::infer::{CodelSettings, InferCodelWidth};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::rc::Rc;
use std::{env, fmt};
use types::color::{Lightness, Lightness::*};
use types::flow::{EntryDir, ExitDir, FindAdj, FURTHEST, MOVE_IN};
use types::instruction::Instruction;
use types::program::PietSource;
use types::state::{Position, ENTRY};
use types::flow::DIRECTIONS;

pub type Node = Rc<ColorBlock>;
pub type Info = Vec<(EntryDir, ExitDir, Option<Instruction>)>;
pub type NodeAdj = HashMap<Node, Info>;
pub type CFG = HashMap<Node, NodeAdj>;

#[allow(unused)]
#[derive(Eq)]
pub struct ColorBlock {
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

    pub fn contains(&self, pos: Position) -> bool {
        self.region.contains(&pos)
    }

    pub fn get_region(&self) -> &HashSet<Position> {
        &self.region
    }

    pub fn get_label(&self) -> &String {
        &self.label
    }

    pub fn get_region_size(&self) -> u64 {
        self.region.len() as u64
    }

    pub fn get_lightness(&self) -> Lightness {
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


#[allow(unused)]
pub struct CFGBuilder<'a> {
    source: &'a PietSource<'a>,
    cfg: CFG,
    codel_width: u32,
}

impl<'a> DecodeInstruction for CFGBuilder<'a> {}
impl<'a> FindAdj for CFGBuilder<'a> {}
impl<'a> InferCodelWidth for CFGBuilder<'a> {}

#[allow(unused)]
impl<'a> CFGBuilder<'a> {
    // Returns the list of adjacencies for a given position and whether or not it is a boundary
    pub fn new(source: &'a PietSource, codel_settings: CodelSettings, show_codel_size: bool) -> Self {
        let codel_width = match codel_settings {
            CodelSettings::Default => 1,
            CodelSettings::Infer => Self::infer_codel_width(source),
            CodelSettings::Width(codel_width) => codel_width,
        };

        if show_codel_size {
            match env::consts::OS {
                "linux" => {
                    println!("\x1B[1;37mpietcc:\x1B[0m \x1B[1;96minfo: \x1B[0mcompiling with codel width {}", codel_width)
                }
                _ => {
                    println!("pietcc: info: compiling with codel width {}", codel_width)
                }
            }
        }

        CFGBuilder {
            source,
            cfg: HashMap::new(),
            codel_width,
        }
    }

    pub fn get_source(&self) -> &'a PietSource<'a> {
        self.source
    }

    fn possible_exits(&self, cb: &HashSet<Position>) -> Vec<(Position, ExitDir)> {
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
                let lightness = self.source.get(pos);
                lightness.is_some() && lightness.unwrap() != &Black
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn trace_white(
        &self,
        entry: Position,
        dir: EntryDir,
    ) -> Option<(Position, ExitDir)> {
        let (mut x, mut y) = entry;
        let (mut dp, mut cc) = dir;
        let mut retries = 0;

        while retries < 8 {
            let next_pos = Some((x, y, self.codel_width))
                .map(MOVE_IN[dp as usize])
                .unwrap();

            let lightness = self.source.get(next_pos);

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
        if let Some(block) = self.cfg.keys().find(|cb| cb.contains(entry)) {
            return block.clone();
        }

        let mut discovered: HashSet<Position> = HashSet::from([entry]);
        let mut queue: VecDeque<Position> = VecDeque::from([entry]);
        let lightness = *self.source.get(entry).unwrap();

        while !queue.is_empty() {
            let pos = queue.pop_front().unwrap();
            let adjs = Self::adjacencies(pos, &self.source, self.codel_width);

            let in_block = adjs
                .iter()
                .filter(|&&pos| *self.source.get(pos).unwrap() == lightness)
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

    pub fn build(&mut self) {
        let init_block = self.explore_region(ENTRY);
        let mut discovered_regions = HashSet::from([init_block.clone()]);
        let mut queue = VecDeque::<Rc<ColorBlock>>::from([init_block]);

        while !queue.is_empty() {
            let curr_block = queue.pop_front().unwrap();
            let curr_exits = self.possible_exits(curr_block.get_region());
            let mut bordering = NodeAdj::new();

            discovered_regions.insert(curr_block.clone());

            for (boundary, dir) in curr_exits {
                let adj_block = self.explore_region(boundary);
                let instr =
                    Self::decode_instr(curr_block.get_lightness(), adj_block.get_lightness());

                match adj_block.get_lightness() {
                    White => {
                        let exit_state = self.trace_white(boundary, dir);
                        if let Some((next_pos, next_dir)) = exit_state {
                            let white_adj_lightness =
                                self.source.get(next_pos).map(|lightness| *lightness);
    
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
                    },
                    _ => {
                        bordering
                            .entry(adj_block.clone())
                            .and_modify(|vec| vec.push((dir, dir, instr)))
                            .or_insert(Vec::from([(dir, dir, instr)]));
                    }
                }

                if !discovered_regions.contains(&adj_block) {
                    discovered_regions.insert(adj_block.clone());
                    queue.push_back(adj_block)
                }
            }
            if curr_block.get_lightness() != White {
                self.cfg.insert(curr_block, bordering);
            }
        }
    }

    pub fn get_state(&self) -> &Self {
        &self
    }

    pub fn get_cfg(&self) -> CFG {
        self.cfg.clone()
    }

    // We can't determine whether an arbitrary Piet program halts since Piet is Turing-complete, which makes this equivalent to solving the halting problem.
    // However, one condition in which a Piet program is guaranteed to run forever is if there are no nodes with outdegree zero.
    // This is because our compilation procedure inserts a return for any such node, which is the only way for termination to occur.
    pub fn determine_runs_forever(&self) -> bool {
        self.cfg.iter().all(|(_, y)| y.len() > 0)
    }
}

#[cfg(Test)]
mod test {
    use super::*;
    use crate::{convert::UnknownPixelSettings, loader::Loader};
    use std::{
        collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
        fs,
        hash::Hasher,
    };

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
        let prog = Loader::convert("../images/hw1-1.png", SETTINGS).unwrap();
        let mut cfg_gen = CFGBuilder::new(&prog, CodelSettings::Width(1), true);

        println!("loaded");
        cfg_gen.build();

        let adjacencies = &cfg_gen.get_state().cfg;

        for node in adjacencies.keys() {
            if node.get_label() == "RegYellow_2_3" {
                println!("{:?}", adjacencies.get(node))
            }
        }
    }
}
