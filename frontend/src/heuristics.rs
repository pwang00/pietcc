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

// We can't determine whether an arbitrary Piet program halts since Piet is Turing-complete,
// which makes this equivalent to solving the halting problem.  However, one condition in which a
// Piet program is guaranteed to run forever is if there are no blocks with outdegree zero, since
// our compilation procedure inserts a return for any such block, which is the only way for
// termination to occur.
pub fn check_nontermination(cfg: &CFG) -> bool {
    cfg.ids().all(|id| !cfg.adjacencies(id).is_empty())
}
