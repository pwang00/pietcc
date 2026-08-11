use crate::color::Lightness;
use crate::flow::PietTransition;
use std::fmt;

/// A color block's identity within a `CFG`, and its index into the graph's side tables.
///
/// Ids are dense and assigned in discovery order, so `BlockId(0)` is always the block the
/// instruction pointer starts in.  Identity being an index rather than a name is what lets
/// lookups be array indexing instead of string hashing, and lets a node be copied rather
/// than cloned.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub usize);

impl BlockId {
    pub const fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

/// The transitions out of one block that land in the same neighbor, ordered by the DP / CC
/// state that takes them.
pub type Transitions = Vec<PietTransition>;

/// One block's neighbors, in the order the exits were discovered.
pub type NodeAdj = Vec<(BlockId, Transitions)>;

/// A color block's payload.  `label` is presentation only — IR basic block names and
/// diagnostics — and carries no identity.
#[derive(Debug, Clone)]
pub struct BlockData {
    pub lightness: Lightness,
    /// Size of the block in codels at the graph's current codel size, which is the value
    /// `Push` pushes.  See [`CFG::reinterpret`].
    pub count: usize,
    /// Bounding box of the block, also in codels at the current codel size.
    pub width: usize,
    pub height: usize,
    pub label: String,
}

#[derive(Debug)]
pub struct CFG {
    blocks: Vec<BlockData>,
    adjacencies: Vec<NodeAdj>,
    codel_size: usize,
}

impl CFG {
    /// The block the instruction pointer starts in.  Guaranteed to exist in any graph built
    /// from a non-empty source, since it is interned before the walk begins.
    pub const ENTRY: BlockId = BlockId(0);

    pub fn new() -> Self {
        Self {
            blocks: vec![],
            adjacencies: vec![],
            // Counts start out in source codels, i.e. one codel per pixel
            codel_size: 1,
        }
    }

    /// Appends a block with no adjacencies yet and returns its id.
    pub fn push(&mut self, data: BlockData) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(data);
        self.adjacencies.push(NodeAdj::new());
        id
    }

    pub fn set_adjacencies(&mut self, id: BlockId, adjacencies: NodeAdj) {
        self.adjacencies[id.index()] = adjacencies;
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Every block id in ascending order.  Iterating this rather than a hash map is what keeps
    /// emitted IR reproducible between runs.
    pub fn ids(&self) -> impl Iterator<Item = BlockId> + use<> {
        (0..self.blocks.len()).map(BlockId)
    }

    pub fn block(&self, id: BlockId) -> &BlockData {
        &self.blocks[id.index()]
    }

    pub fn adjacencies(&self, id: BlockId) -> &[(BlockId, Transitions)] {
        &self.adjacencies[id.index()]
    }

    pub fn codel_size(&self) -> usize {
        self.codel_size
    }

    /// Rescales every block from the graph's current codel size to `codel_size`.
    ///
    /// The frontend cannot know the codel size until the graph exists, since the heuristic that
    /// infers it needs the blocks' dimensions, so blocks are first measured in source codels
    /// and rescaled once afterwards.  Rescaling relative to the current size rather than the
    /// source keeps repeated calls consistent.  Only sizes change: the set of blocks and the
    /// transitions between them are independent of the codel size.
    pub fn reinterpret(&mut self, codel_size: usize) {
        assert!(codel_size > 0, "codel size must be positive");
        if codel_size == self.codel_size {
            return;
        }

        let (from, to) = (self.codel_size, codel_size);
        for block in &mut self.blocks {
            block.count = block.count * from * from / (to * to);
            block.width = block.width * from / to;
            block.height = block.height * from / to;
        }

        self.codel_size = codel_size;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::color::{Hue::Red, Lightness::Reg};

    fn block(count: usize, width: usize, height: usize) -> BlockData {
        BlockData {
            lightness: Reg(Red),
            count,
            width,
            height,
            label: format!("RegRed_{width}_{height}"),
        }
    }

    #[test]
    fn test_push_hands_out_dense_ascending_ids() {
        let mut cfg = CFG::new();

        assert_eq!(cfg.push(block(1, 1, 1)), CFG::ENTRY);
        assert_eq!(cfg.push(block(1, 1, 1)).index(), 1);
        assert_eq!(cfg.len(), 2);
        assert_eq!(cfg.ids().map(|id| id.index()).collect::<Vec<_>>(), [0, 1]);
        // Pushing leaves a block with no exits, which is how termination is spelled
        assert!(cfg.adjacencies(CFG::ENTRY).is_empty());
    }

    #[test]
    fn test_reinterpret_rescales_sizes_by_codel_area() {
        let mut cfg = CFG::new();
        cfg.push(block(16, 4, 4));

        cfg.reinterpret(2);

        let scaled = cfg.block(CFG::ENTRY);
        assert_eq!(cfg.codel_size(), 2);
        // 16 source codels of area 4 is 4 codels, in a 2x2 box
        assert_eq!(scaled.count, 4);
        assert_eq!((scaled.width, scaled.height), (2, 2));
    }

    #[test]
    fn test_reinterpret_is_relative_to_the_current_size() {
        let mut once = CFG::new();
        once.push(block(400, 20, 20));
        once.reinterpret(2);

        // Going by way of another size has to land in the same place, or a second call would
        // rescale sizes that were already rescaled
        let mut twice = CFG::new();
        twice.push(block(400, 20, 20));
        twice.reinterpret(4);
        twice.reinterpret(2);

        assert_eq!(twice.block(CFG::ENTRY).count, once.block(CFG::ENTRY).count);
        assert_eq!(twice.block(CFG::ENTRY).width, once.block(CFG::ENTRY).width);
        assert_eq!(twice.codel_size(), once.codel_size());
    }

    #[test]
    fn test_reinterpret_to_the_same_size_is_a_noop() {
        let mut cfg = CFG::new();
        cfg.push(block(9, 3, 3));

        cfg.reinterpret(1);

        assert_eq!(cfg.block(CFG::ENTRY).count, 9);
    }

    #[test]
    #[should_panic(expected = "codel size must be positive")]
    fn test_reinterpret_rejects_zero() {
        let mut cfg = CFG::new();
        cfg.push(block(9, 3, 3));
        cfg.reinterpret(0);
    }
}
