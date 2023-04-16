use crate::color::Lightness;
use crate::state::Position;

pub struct Program<'a> {
    height: u32,
    width: u32,
    prog: &'a Vec<Lightness>,
}

impl<'a> Program<'a> {
    pub fn new(prog: &'a Vec<Lightness>, height: u32, width: u32) -> Self {
        Program {
            height,
            width,
            prog,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.height, self.width)
    }

    pub fn get_underlying_vec(&self) -> &Vec<Lightness> {
        self.prog
    }

    pub fn get(&self, (r, c): Position) -> Option<&'a Lightness> {
        self.prog
            .get(self.width.wrapping_mul(r).wrapping_add(c) as usize)
            .filter(|_| r < self.height as u32 && c < self.width)
    }
}

#[allow(unused)]
mod test {
    use super::*;
    use crate::color::Hue::*;
    #[cfg(test)]
    use crate::color::Lightness::*;
    #[test]
    fn test_program_get() {
        let vec = vec![
            Light(Red),
            Reg(Red),
            Dark(Red),
            Light(Blue),
            Reg(Blue),
            Dark(Blue),
            Light(Magenta),
            Reg(Magenta),
            Dark(Magenta),
        ];

        let prog = Program::new(&vec, 3, 3);
        let pos1 = (1, 2);
        let pos2 = (0, 2);
        let pos3 = (2, 1);
        let pos4 = (2, 3);
        assert_eq!(prog.get(pos1), Some(&Dark(Blue)));
        assert_eq!(prog.get(pos2), Some(&Dark(Red)));
        assert_eq!(prog.get(pos3), Some(&Reg(Magenta)));
        assert_eq!(prog.get(pos4), None);
    }
}
