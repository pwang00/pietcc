use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use crate::color::Lightness;
use crate::flow::PietTransition;
use crate::state::Position;

pub type Node = Rc<ColorBlock>;
pub type Info = Vec<PietTransition>;
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
