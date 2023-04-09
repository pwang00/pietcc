use crate::{program::Program, state::Position};

type C1 = fn(&&(u32, u32)) -> (i64, i64);
type C2 = fn((u32, u32, u32)) -> (u32, u32);

pub const FURTHEST: [C1; 8] = [
    |&&(x, y)| (y as i64, -(x as i64)),    // dp = right, cc = left
    |&&(x, y)| (y as i64, x as i64),       // dp = right, cc = right
    |&&(x, y)| (x as i64, y as i64),       // dp = down, cc = left
    |&&(x, y)| (x as i64, -(y as i64)),    // dp = down, cc = right
    |&&(x, y)| (-(y as i64), x as i64),    // dp = left, cc = left
    |&&(x, y)| (-(y as i64), -(x as i64)), // dp = left, cc = right
    |&&(x, y)| (-(x as i64), -(y as i64)), // dp = up, cc = left
    |&&(x, y)| (-(x as i64), y as i64),    // dp = up, cc = right
];

pub const MOVE_IN: [C2; 4] = [
    |(x, y, cs)| (x, y.wrapping_add(cs)),
    |(x, y, cs)| (x.wrapping_add(cs), y),
    |(x, y, cs)| (x, y.wrapping_sub(cs)),
    |(x, y, cs)| (x.wrapping_sub(cs), y),
];

#[derive(Debug, PartialEq, Default, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum Direction {
    #[default]
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

#[derive(Debug, PartialEq, Default, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum Codel {
    #[default]
    Left,
    Right,
}

impl Direction {
    pub fn rotate(self, n: i64) -> Self {
        <Self as DirectionOps>::from_idx(self as i64 + n)
    }
}

impl Codel {
    pub fn switch(self, n: i64) -> Self {
        <Codel as DirectionOps>::from_idx(self as i64 + n)
    }
}

pub type DirVec = (Direction, Codel);

pub trait DirectionOps {
    fn from_idx(i: i64) -> Self;
}

impl DirectionOps for Direction {
    fn from_idx(i: i64) -> Self {
        match i {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            i => <Direction as DirectionOps>::from_idx(i.rem_euclid(4)),
        }
    }
}

impl DirectionOps for Codel {
    fn from_idx(i: i64) -> Self {
        match i {
            0 => Codel::Left,
            1 => Codel::Right,
            i => <Codel as DirectionOps>::from_idx(i % 2),
        }
    }
}

pub trait FindAdj {
    fn adjacencies(&self, (r, c): Position, program: &Program, cs: u32) -> Vec<Position> {
        vec![
            (r.wrapping_add(cs), c),
            (r.wrapping_sub(cs), c),
            (r, c.wrapping_add(cs)),
            (r, c.wrapping_sub(cs)),
        ]
        .iter()
        .filter_map(|&pos| program.get(pos).map(|_| pos))
        .collect()
    }
}
