use gcd::Gcd;
use piet_core::cfg::{BlockId, CFG};
use std::usize;

// The idea of this heuristic is that since the codel size is constant across the program,
// each block must have width and height as a multiple of the codel size.  A somewhat reasonable metric
// is to compute the n-ary gcd of all block widths and height.
pub(crate) fn nary_gcd(cfg: &CFG) -> usize {
    cfg.ids()
        .map(|id| cfg.block(id))
        .fold(cfg.block(BlockId(0)).width, |acc, block| {
            acc.gcd(block.width).gcd(block.height)
        })
}
