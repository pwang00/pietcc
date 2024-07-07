use std::collections::HashSet;

use crate::{program::PietSource, state::Position};

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
    |(x, y, cs)| (x, y.wrapping_add(cs)), // dp = right
    |(x, y, cs)| (x.wrapping_add(cs), y), // dp = down
    |(x, y, cs)| (x, y.wrapping_sub(cs)), // dp = left
    |(x, y, cs)| (x.wrapping_sub(cs), y), // dp = up
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
    Left = 0,
    Right = 1,
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

impl std::ops::Sub for Direction {
    type Output = u8;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 - rhs as u8).rem_euclid(4)
    }
}

impl std::ops::Sub for Codel {
    type Output = u8;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 - rhs as u8).rem_euclid(2)
    }
}

// (Up, Right) => (Right, Left)
pub fn find_offset(curr: DirVec, target: DirVec) -> u8 {
    let curr_idx = 2 * curr.0 as u8 + curr.1 as u8;
    let target_idx = 2 * target.0 as u8 + target.1 as u8;

    std::cmp::min((curr_idx - target_idx).rem_euclid(8), (target_idx - curr_idx).rem_euclid(8))
}

pub type DirVec = (Direction, Codel);
pub type EntryDir = (Direction, Codel);
pub type ExitDir = (Direction, Codel);

pub const DIRECTIONS: [DirVec; 8] = [
    (Direction::Right, Codel::Left),
    (Direction::Right, Codel::Right),
    (Direction::Down, Codel::Left),
    (Direction::Down, Codel::Right),
    (Direction::Left, Codel::Left),
    (Direction::Left, Codel::Right),
    (Direction::Up, Codel::Left),
    (Direction::Up, Codel::Right),
];

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
            i => <Codel as DirectionOps>::from_idx(i.rem_euclid(2)),
        }
    }
}

pub trait FindAdj {
    fn adjacencies((r, c): Position, program: &PietSource, cs: u32) -> HashSet<Position> {
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
