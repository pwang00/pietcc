use crate::color::Lightness;
use crate::state::Position;

pub type PietSource = Raster<Lightness>;

pub struct Raster<T: Copy> {
    height: usize,
    width: usize,
    grid: Vec<T>,
}

impl<T: Copy> Raster<T> {
    pub fn new(grid: Vec<T>, height: usize, width: usize) -> Self {
        assert_eq!(grid.len(), height * width);

        Self {
            height,
            width,
            grid,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    pub fn get(&self, (r, c): Position) -> Option<T> {
        if r >= self.height || c >= self.width {
            return None;
        }
        self.grid.get(r * self.width + c).copied()
    }

    pub fn set(&mut self, (r, c): Position, val: T) -> bool {
        if r >= self.height || c >= self.width {
            return false;
        }
        self.grid[r * self.width + c] = val;
        true
    }
}

#[cfg(test)]
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

        let prog = PietSource::new(vec, 3, 3);
        let pos1 = (1, 2);
        let pos2 = (0, 2);
        let pos3 = (2, 1);
        let pos4 = (2, 3);
        assert_eq!(prog.get(pos1), Some(Dark(Blue)));
        assert_eq!(prog.get(pos2), Some(Dark(Red)));
        assert_eq!(prog.get(pos3), Some(Reg(Magenta)));
        assert_eq!(prog.get(pos4), None);
    }
}
