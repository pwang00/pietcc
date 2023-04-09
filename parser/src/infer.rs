use gcd::Gcd;
use std::collections::HashSet;
use types::program::Program;

pub trait InferCodelSize {
    fn infer_codel_size(program: &Program) -> u32 {
        let vec = program.get_underlying_vec();
        let set = vec.iter().collect::<HashSet<_>>();

        set.iter()
            .map(&&|color| vec.iter().filter(|x| x == color).count() as u32)
            .reduce(|x, y| x.gcd(y))
            .unwrap()
    }
}
