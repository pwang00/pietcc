use crate::structures::DisjointSet;
use piet_core::{program::PietSource, state::Position};

/// Implements two-pass CCL: https://en.wikipedia.org/wiki/Connected-component_labeling
pub fn lex(source: PietSource) -> DisjointSet {
    let mut uf = DisjointSet::new(source);
    let (h, w) = uf.source.dimensions();

    for i in 0..h {
        for j in 0..w {
            match get_adjacent_classes((i, j), &uf)[..] {
                // No same colored neighbor above or to the left, so this codel starts a class
                [] => {
                    let new_class = uf.new_equiv_class();
                    uf.labels.set((i, j), new_class);
                }
                // Otherwise the codel joins its neighbors, merging their classes if they
                // were discovered separately.  The label is resolved to a root below and written to the label raster
                [first, ref rest @ ..] => {
                    rest.iter().for_each(|&other| uf.union(first, other));
                    uf.labels.set((i, j), first);
                }
            }
        }
    }

    for i in 0..h {
        for j in 0..w {
            let eq = uf.find(uf.labels.get((i, j)).unwrap());
            uf.labels.set((i, j), eq);
        }
    }

    uf
}

/// Fetches adjacencies of the same color
#[inline]
fn get_adjacent_classes((r, c): Position, uf: &DisjointSet) -> Vec<usize> {
    let lightness_at_pos = uf.source.get((r, c)).unwrap();
    [(r.wrapping_sub(1), c), (r, c.wrapping_sub(1))]
        .iter()
        .filter_map(|&pos| {
            if let Some(lightness) = uf.source.get(pos)
                && lightness == lightness_at_pos
            {
                Some(uf.labels.get(pos).unwrap())
            } else {
                None
            }
        })
        .collect()
}
