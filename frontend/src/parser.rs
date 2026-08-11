use crate::conversions::decode_instr;
use crate::structures::{ComponentMetadata, DisjointSet};
use piet_core::cfg::{BlockData, BlockId, CFG, NodeAdj};
use piet_core::color::Lightness::{Black, White};
use piet_core::flow::{DIRECTIONS, DirPointer, FURTHEST, MOVE_IN, PietTransition, PointerState};
use piet_core::program::PietSource;
use piet_core::state::{ENTRY, Position};
use std::collections::VecDeque;

/// The eight exits also encode a block's bounding box, since each `FURTHEST` key extremizes
/// one edge: index 0 maximizes the column, 2 the row, 4 minimizes the column, and 6 minimizes
/// the row.  Index 6 breaks its tie leftward, making it the topmost then leftmost codel, which
/// is what anchors the block's label.
const RIGHTMOST: usize = 0;
const BOTTOMMOST: usize = 2;
const LEFTMOST: usize = 4;
const ANCHOR: usize = 6;

fn update_exits(block: &mut ComponentMetadata, pos: Position) {
    block
        .exits
        .iter_mut()
        .enumerate()
        .for_each(|(i, exit)| *exit = std::cmp::max_by_key(*exit, pos, FURTHEST[i]));
}

/// Steps a single codel out of `pos` along `dp`.  Color blocks are codel aligned, so one
/// pixel is enough to leave the block no matter the codel width, and an underflow wraps
/// out of the raster's bounds, where `Raster::get` reports it as off canvas.
fn step_out((r, c): Position, dp: DirPointer) -> Position {
    let (r, c) = MOVE_IN[dp as usize]((r, c, 1));
    (r as usize, c as usize)
}

/// Slides through the white region entered at `entry`, returning the first non-white codel
/// reached along with the pointer state on arrival. Semantics defined here: https://www.dangermouse.net/esoteric/piet.html
fn trace_white(
    source: &PietSource,
    entry: Position,
    dir: PointerState,
) -> Option<(Position, PointerState)> {
    let (mut r, mut c) = entry;
    let mut dp = dir.dp;
    let mut cc = dir.cc;
    let mut retries = 0;

    while retries < 8 {
        let next = step_out((r, c), dp);

        match source.get(next) {
            // Blocked by the edge of the canvas or by a black block: turn and retry from here
            None | Some(Black) => {
                dp = dp.rotate(1);
                cc = cc.switch(1);
                retries += 1;
            }
            // Still white, so keep sliding
            Some(White) => (r, c) = next,
            Some(_) => return Some((next, PointerState::new(dp, cc))),
        }
    }

    None
}

/// Builds a component's graph payload.  Sizes are in source codels; the pipeline rescales them
/// once the codel size is known.
fn block_data(block: &ComponentMetadata) -> BlockData {
    let lightness = block.lightness.expect("component has no codels");
    let (r, c) = block.exits[ANCHOR];

    BlockData {
        lightness,
        count: block.count,
        width: block.exits[RIGHTMOST].1 - block.exits[LEFTMOST].1 + 1,
        height: block.exits[BOTTOMMOST].0 - block.exits[ANCHOR].0 + 1,
        label: format!("{}_{}_{}", lightness.to_string(), r, c),
    }
}

/// Resolves a component to its dense `BlockId`, interning it and queueing it for exploration
/// the first time it is reached.  Ids are handed out in discovery order, so the entry component
/// — interned before the walk starts — is always `CFG::ENTRY`.
fn intern(
    root: usize,
    blocks: &[ComponentMetadata],
    ids: &mut [Option<BlockId>],
    queue: &mut VecDeque<usize>,
    cfg: &mut CFG,
) -> BlockId {
    match ids[root] {
        Some(id) => id,
        None => {
            let id = cfg.push(block_data(&blocks[root]));
            ids[root] = Some(id);
            queue.push_back(root);
            id
        }
    }
}

/// Lifts the labeled raster produced during lexing into a control flow graph.
///
/// The raster sweep collects each component's color, size in pixels, and furthest codel per DP / CC
/// state; the walk then steps one codel out of those exits to find the blocks the IP can
/// reach, emitting each block's transitions in `FURTHEST` order.
pub(crate) fn parse(mut uf: DisjointSet) -> CFG {
    let (h, w) = uf.source.dimensions();
    let mut blocks = vec![ComponentMetadata::default(); uf.nodes.len()];

    // Traverse the raster, setting color blocks
    for i in 0..h {
        for j in 0..w {
            let pos = (i, j);
            let root = uf.find(uf.labels.get(pos).unwrap());
            uf.labels.set(pos, root);

            let block = &mut blocks[root];
            block.count += 1;

            // The exits cannot start at their default of (0, 0), which wins several of the
            // FURTHEST keys outright, so seed them with the component's first codel instead.
            if block.lightness.is_none() {
                block.lightness = uf.source.get(pos);
                block.exits = [pos; 8];
            } else {
                update_exits(block, pos);
            }
        }
    }

    let mut cfg = CFG::new();
    let mut ids = vec![None; blocks.len()];
    let mut queue = VecDeque::new();

    // The IP starts in the block holding the top left codel.  If that block is white it slides
    // first, and the block it slides into stands in as the entry node.
    let start = uf.labels.get(ENTRY).unwrap();
    let entry = match blocks[start].lightness {
        Some(White) => trace_white(&uf.source, ENTRY, PointerState::default())
            .map(|(pos, _)| uf.labels.get(pos).unwrap())
            .unwrap_or(start),
        _ => start,
    };

    // Interning the entry component first is what makes it CFG::ENTRY
    intern(entry, &blocks, &mut ids, &mut queue, &mut cfg);

    // White / black block early termination optimization
    // White blocks would have been merged earlier, so if we see it again then this means we traced and found no valid exits
    if matches!(blocks[entry].lightness, Some(White | Black)) {
        return cfg;
    }

    while let Some(root) = queue.pop_front() {
        let id = ids[root].expect("queued components are interned");
        let lightness = blocks[root].lightness.expect("component has no codels");
        let mut adjacencies = NodeAdj::new();

        // DIRECTIONS shares its indices with FURTHEST, so walking it in order emits the
        // transitions in DP / CC order as well.
        for (i, &dir) in DIRECTIONS.iter().enumerate() {
            let boundary = step_out(blocks[root].exits[i], dir.dp);

            let Some(bordering) = uf.source.get(boundary) else {
                continue;
            };

            if bordering == Black {
                continue;
            }

            let (adj_root, transition) = if bordering == White {
                match trace_white(&uf.source, boundary, dir) {
                    // Passing through white leaves the pointers wherever the slide ended and
                    // executes no instruction
                    Some((pos, exit_state)) => (
                        uf.labels.get(pos).unwrap(),
                        PietTransition::new(dir, exit_state, None),
                    ),
                    None => continue,
                }
            } else {
                let instruction = decode_instr(lightness, bordering);
                (
                    uf.labels.get(boundary).unwrap(),
                    PietTransition::new(dir, dir, instruction),
                )
            };

            let adj_id = intern(adj_root, &blocks, &mut ids, &mut queue, &mut cfg);

            // At most eight entries, so a scan beats hashing and keeps the exits in DP / CC
            // order within each neighbor.
            match adjacencies.iter_mut().find(|(id, _)| *id == adj_id) {
                Some((_, transitions)) => transitions.push(transition),
                None => adjacencies.push((adj_id, vec![transition])),
            }
        }

        cfg.set_adjacencies(id, adjacencies);
    }

    cfg
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conversions::UnknownPixelSettings;
    use crate::load::to_lightness_raster;
    use piet_core::color::Hue::*;
    use piet_core::color::Lightness::{self, *};
    use piet_core::flow::{CodelChooser, DirPointer};
    use piet_core::instruction::Instruction;
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    const SETTINGS: UnknownPixelSettings = UnknownPixelSettings::TreatAsError;

    fn get_hash<T: Hash>(val: &T) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    /// Lexes and parses a raster given a codel per row entry.
    fn cfg_from(rows: &[&[Lightness]]) -> CFG {
        let (h, w) = (rows.len(), rows[0].len());
        let grid = rows.concat();
        parse(lex(PietSource::new(grid, h, w)))
    }

    fn node_by_label<'a>(cfg: &'a CFG, label: &str) -> &'a ColorBlock {
        cfg.keys()
            .find(|node| node.label == label)
            .unwrap_or_else(|| panic!("no node labeled {label} in {:?}", cfg.keys()))
    }

    /// The transitions out of `label`, keyed by the DP / CC state that takes them.
    fn transitions_by_entry(
        cfg: &CFG,
        label: &str,
    ) -> HashMap<(DirPointer, CodelChooser), (String, Option<Instruction>, PointerState)> {
        let mut by_entry = HashMap::new();

        for (adj, transitions) in cfg.get(node_by_label(cfg, label)).unwrap() {
            for transition in transitions {
                let entry = transition.entry_state;
                let prev = by_entry.insert(
                    (entry.dp, entry.cc),
                    (
                        adj.label.clone(),
                        transition.instruction,
                        transition.exit_state,
                    ),
                );
                assert!(
                    prev.is_none(),
                    "{label} has two transitions for dp {:?} / cc {:?}",
                    entry.dp,
                    entry.cc
                );
            }
        }

        by_entry
    }

    #[test]
    fn test_colorblock_eq_hash() {
        let cb1 = ColorBlock::new(String::from("LightRed_1_2"), Light(Red), 8);
        let cb2 = ColorBlock::new(String::from("LightRed_1_2"), Light(Red), 8);

        assert_eq!(cb1, cb2);
        assert_eq!(get_hash(&cb1), get_hash(&cb2));
    }

    #[test]
    fn test_single_block_program_terminates() {
        let cfg = cfg_from(&[&[Reg(Red)]]);

        assert_eq!(cfg.len(), 1);
        let entry = node_by_label(&cfg, "Entry");
        assert_eq!(entry.count, 1);
        // Every exit runs off the canvas, so the block has no adjacencies and the program ends
        assert!(cfg.get(entry).unwrap().is_empty());
    }

    #[test]
    fn test_adjacent_blocks_decode_instruction() {
        let cfg = cfg_from(&[&[Reg(Red), Reg(Yellow)]]);

        assert_eq!(cfg.len(), 2);

        // One hue step right and no lightness change is Add, and the reverse is CharIn
        let entry = transitions_by_entry(&cfg, "Entry");
        assert_eq!(entry.len(), 2);
        for cc in [CodelChooser::Left, CodelChooser::Right] {
            let (label, instruction, _) = &entry[&(DirPointer::Right, cc)];
            assert_eq!(label, "RegYellow_0_1");
            assert_eq!(*instruction, Some(Instruction::Add));
        }

        let yellow = transitions_by_entry(&cfg, "RegYellow_0_1");
        assert_eq!(yellow.len(), 2);
        for cc in [CodelChooser::Left, CodelChooser::Right] {
            let (label, instruction, _) = &yellow[&(DirPointer::Left, cc)];
            assert_eq!(label, "Entry");
            assert_eq!(*instruction, Some(Instruction::CharIn));
        }
    }

    #[test]
    fn test_exits_track_furthest_codel_per_state() {
        // A 2x2 red block whose right edge borders a different block per row, so dp = right
        // reaches yellow under cc = left and green under cc = right.
        let cfg = cfg_from(&[
            &[Reg(Red), Reg(Red), Reg(Yellow)],
            &[Reg(Red), Reg(Red), Reg(Green)],
        ]);

        assert_eq!(cfg.len(), 3);
        assert_eq!(node_by_label(&cfg, "Entry").count, 4);

        let entry = transitions_by_entry(&cfg, "Entry");
        // Only the two right facing exits clear the block; the rest run off the canvas
        assert_eq!(entry.len(), 2);

        let (label, instruction, _) = &entry[&(DirPointer::Right, CodelChooser::Left)];
        assert_eq!(label, "RegYellow_0_2");
        assert_eq!(*instruction, Some(Instruction::Add));

        let (label, instruction, _) = &entry[&(DirPointer::Right, CodelChooser::Right)];
        assert_eq!(label, "RegGreen_1_2");
        assert_eq!(*instruction, Some(Instruction::Div));
    }

    #[test]
    fn test_white_regions_are_traced_through() {
        let cfg = cfg_from(&[&[Reg(Red), White, White, Reg(Green)]]);

        // White is passed through rather than becoming a node of its own
        assert_eq!(cfg.len(), 2);
        assert!(cfg.keys().all(|node| node.lightness != White));

        let entry = transitions_by_entry(&cfg, "Entry");
        let (label, instruction, exit_state) = &entry[&(DirPointer::Right, CodelChooser::Left)];
        assert_eq!(label, "RegGreen_0_3");
        // Sliding through white executes no instruction and leaves the pointers untouched
        assert_eq!(*instruction, None);
        assert_eq!(
            (exit_state.dp, exit_state.cc),
            (DirPointer::Right, CodelChooser::Left)
        );
    }

    #[test]
    fn test_white_slide_turns_at_obstacles() {
        // Entering white at (0, 1) heading right is blocked by black, so the slide turns
        // clockwise to dp = down (toggling cc) and leaves the white region into green.
        let cfg = cfg_from(&[&[Reg(Red), White, Black], &[Black, Reg(Green), Black]]);

        let entry = transitions_by_entry(&cfg, "Entry");
        let (label, instruction, exit_state) = &entry[&(DirPointer::Right, CodelChooser::Left)];
        assert_eq!(label, "RegGreen_1_1");
        assert_eq!(*instruction, None);
        assert_eq!(
            (exit_state.dp, exit_state.cc),
            (DirPointer::Down, CodelChooser::Right)
        );
    }

    #[test]
    fn test_black_blocks_are_not_reachable() {
        let cfg = cfg_from(&[&[Reg(Red), Black, Reg(Green)]]);

        // Black is never an adjacency, so green is unreachable and never discovered
        assert_eq!(cfg.len(), 1);
        assert!(cfg.get(node_by_label(&cfg, "Entry")).unwrap().is_empty());
    }

    #[test]
    fn test_components_merge_across_scan_lines() {
        // The two red runs on the top row are labeled separately and only turn out to be one
        // component when the bottom row joins them.
        let cfg = cfg_from(&[
            &[Reg(Red), Black, Reg(Red)],
            &[Reg(Red), Reg(Red), Reg(Red)],
        ]);

        assert_eq!(cfg.len(), 1);
        assert_eq!(node_by_label(&cfg, "Entry").count, 5);
    }

    #[test]
    fn test_program_graph_is_well_formed() {
        let source = to_lightness_raster("../images/hw1-1.png", SETTINGS).unwrap();
        let cfg = CFGBuilder::build(source, CodelSettings::Width(1), false);

        assert!(!cfg.is_empty());
        node_by_label(&cfg, "Entry");

        for node in cfg.keys() {
            // The IR builder resolves every adjacency to a basic block, so each one has to be
            // a node of the graph in its own right, and no node may be white.
            assert!(node.lightness != White, "{} is a white node", node.label);
            assert!(node.count > 0);

            // Also asserts that no DP / CC state has two transitions out of a node
            transitions_by_entry(&cfg, &node.label);

            for adj in cfg.get(node).unwrap().keys() {
                assert!(
                    cfg.contains_key(adj),
                    "{} is an adjacency of {} but not a node",
                    adj.label,
                    node.label
                );
            }
        }
    }
}
