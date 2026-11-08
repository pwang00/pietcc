use piet_core::{cfg::CFG, program::PietSource, settings::CodelSettings};

use crate::heuristics::nary_gcd;
use crate::lexer::lex;
use crate::parser::parse;

/// Pipeline steps:
/// 1. Lexes the Piet source using two-pass CCL via union-find, which produces a label raster
///    containing the partitions of the source into equivalence classes
///
/// 2. Parses the resulting CCL label raster into color blocks, building exits for each block
///    incrementally and then computes Piet-specific adjacencies based on dp / cc state
///
/// 3. Runs any relevant heuristics on the resulting CFG to determine codel size, and
///    reinterprets the count accordingly.
pub fn run_frontend_pipeline(source: PietSource, codel_settings: CodelSettings) -> CFG {
    let mut cfg = parse(lex(source));

    let codel_size = match codel_settings {
        CodelSettings::Default => 1,
        CodelSettings::Infer => nary_gcd(&cfg),
        CodelSettings::Width(cs) => cs,
    };

    // A codel size of zero means the heuristic could not settle on one, in which case the
    // source is already its own codel grid and the counts parse produced stand.
    cfg.reinterpret(codel_size.max(1));
    cfg
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conversions::{UnknownPixelSettings, rgb_to_lightness};

    fn load(path: &str) -> PietSource {
        let img = image::open(path).unwrap().into_rgb8();
        let (w, h) = img.dimensions();
        PietSource::new(
            img.pixels()
                .map(|pix| rgb_to_lightness(pix, UnknownPixelSettings::TreatAsError))
                .collect(),
            h as usize,
            w as usize,
        )
    }

    /// Asserts two graphs have the same shape and sizes.  Labels are excluded: their anchors are
    /// source coordinates, so they scale with the codel size even when nothing structural does.
    fn assert_same_shape(a: &CFG, b: &CFG) {
        assert_eq!(a.len(), b.len(), "block counts differ");

        for id in a.ids() {
            let (left, right) = (a.block(id), b.block(id));
            assert_eq!(left.lightness, right.lightness, "{id} color");
            assert_eq!(left.count, right.count, "{id} codel count");
            assert_eq!(
                (left.width, left.height),
                (right.width, right.height),
                "{id} bounding box"
            );

            let (adj_left, adj_right) = (a.adjacencies(id), b.adjacencies(id));
            assert_eq!(adj_left.len(), adj_right.len(), "{id} outdegree");

            for ((to_left, via_left), (to_right, via_right)) in adj_left.iter().zip(adj_right) {
                assert_eq!(to_left, to_right, "{id} neighbor");
                assert_eq!(via_left.len(), via_right.len(), "{id} transition count");

                for (l, r) in via_left.iter().zip(via_right) {
                    assert_eq!(
                        (l.entry_state.dp, l.entry_state.cc),
                        (r.entry_state.dp, r.entry_state.cc),
                    );
                    assert_eq!(
                        (l.exit_state.dp, l.exit_state.cc),
                        (r.exit_state.dp, r.exit_state.cc),
                    );
                    assert_eq!(l.instruction, r.instruction);
                }
            }
        }
    }

    #[test]
    fn test_codel_size_changes_sizes_but_not_shape() {
        // test2_upscaled.png is test2.png at 20 codels per codel, and hw5_big.png is hw5.png at
        // 5.  Ids are handed out in discovery order, so identical shapes also means the two
        // graphs number their blocks identically.
        for (small, big, codel_size) in [
            ("../images/test2.png", "../images/test2_upscaled.png", 20),
            ("../images/hw5.png", "../images/hw5_big.png", 5),
        ] {
            let small = run_frontend_pipeline(load(small), CodelSettings::Default);
            let big = run_frontend_pipeline(load(big), CodelSettings::Width(codel_size));

            assert_same_shape(&small, &big);
        }
    }

    #[test]
    fn test_counts_are_source_codels_without_a_codel_size() {
        // Left at the source's own grid, an upscaled program's blocks are codel_size^2 larger,
        // which is what makes the scaling recoverable after the fact
        let small = run_frontend_pipeline(load("../images/test2.png"), CodelSettings::Default);
        let big =
            run_frontend_pipeline(load("../images/test2_upscaled.png"), CodelSettings::Default);

        assert_eq!(big.codel_size(), 1);
        for id in small.ids() {
            assert_eq!(big.block(id).count, small.block(id).count * 400);
        }
    }
}
