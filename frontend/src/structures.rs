use piet_core::color::Lightness;
use piet_core::program::{PietSource, Raster};
use piet_core::state::Position;

// Raster for labeling each pixel with equivalence classes
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct Label {
    parent: usize,
    size: usize,
}

/// Component metadata for the color block to aid in CFG generation during parsing
/// lightness: color of the block
/// count: number of codels in the block
/// exits: all possible Piet-specific exits from the block
///
/// The block's bounding box, which the codel size heuristic needs, is not stored separately:
/// four of the eight exits already extremize one edge of the box each, so `parser` derives the
/// width and height from `exits` rather than tracking them alongside it.
#[derive(Default, Copy, Clone)]
pub(crate) struct ComponentMetadata {
    pub lightness: Option<Lightness>,
    pub count: usize,
    pub exits: [Position; 8],
}

impl Label {
    pub fn new(parent: usize) -> Self {
        Self { parent, size: 1 }
    }
}

pub(crate) struct DisjointSet {
    pub labels: Raster<usize>,
    pub source: PietSource,
    pub nodes: Vec<Label>,
}

impl DisjointSet {
    pub fn new(source: PietSource) -> Self {
        let (h, w) = source.dimensions();
        let grid = vec![0usize; h * w];
        Self {
            labels: Raster::new(grid, h, w),
            source,
            nodes: vec![],
        }
    }

    pub fn new_equiv_class(&mut self) -> usize {
        let class = self.nodes.len();
        self.nodes.push(Label::new(class));
        class
    }

    pub fn find(&mut self, x: usize) -> usize {
        let parent = self.nodes[x].parent;

        if parent != x {
            let root = self.find(parent);
            self.nodes[x].parent = root;
            root
        } else {
            x
        }
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let mut xr = self.find(x);
        let mut yr = self.find(y);

        if xr == yr {
            return;
        }

        if self.nodes[xr].size < self.nodes[yr].size {
            std::mem::swap(&mut xr, &mut yr);
        }

        self.nodes[yr].parent = xr;
        self.nodes[xr].size += self.nodes[yr].size;
    }
}
