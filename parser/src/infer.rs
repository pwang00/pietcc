use gcd::Gcd;
use std::collections::HashSet;
use types::program::Program;

pub trait InferCodelWidth {
    fn infer_codel_width(program: &Program) -> u32 {
        let vec = program.get_underlying_vec();
        let set = vec.iter().collect::<HashSet<_>>();

        if set.len() < 2 {
            return 1
        }

        let width = (set
            .iter()
            .map(&&|color| vec.iter().filter(|x| x == color).count() as u32)
            .reduce(|x, y| x.gcd(y))
            .unwrap() as f64)
            .sqrt() as u32;

        for i in 2..=width as u32 {
            if width % i == 0 {
                return i;
            }
        }
        width
    }
}
