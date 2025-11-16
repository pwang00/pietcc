use std::{collections::HashSet, fmt::Pointer};

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
pub enum DirPointer {
    #[default]
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

#[derive(Debug, PartialEq, Default, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum CodelChooser {
    #[default]
    Left = 0,
    Right = 1,
}

impl DirPointer {
    pub fn rotate(self, n: i64) -> Self {
        <Self as DirectionOps>::from_idx(self as i64 + n)
    }
}

impl CodelChooser {
    pub fn switch(self, n: i64) -> Self {
        <CodelChooser as DirectionOps>::from_idx(self as i64 + n)
    }
}

impl std::ops::Sub for DirPointer {
    type Output = u8;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 - rhs as u8).rem_euclid(4)
    }
}

impl std::ops::Sub for CodelChooser {
    type Output = u8;

    fn sub(self, rhs: Self) -> Self::Output {
        (self as u8 - rhs as u8).rem_euclid(2)
    }
}

// (Up, Right) => (Right, Left)
pub fn find_offset(curr: PointerState, target: PointerState) -> u8 {
    let curr_idx = 2 * curr.dp as u8 + curr.cc as u8;
    let target_idx = 2 * target.dp as u8 + target.cc as u8;

    std::cmp::min(
        (curr_idx - target_idx).rem_euclid(8),
        (target_idx - curr_idx).rem_euclid(8),
    )
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PointerState {
    pub dp: DirPointer,
    pub cc: CodelChooser,
}

impl PointerState {
    pub const fn new(dp: DirPointer, cc: CodelChooser) -> Self {
        Self { dp, cc }
    }
}

pub const DIRECTIONS: [PointerState; 8] = [
    PointerState::new(DirPointer::Right, CodelChooser::Left),
    PointerState::new(DirPointer::Right, CodelChooser::Right),
    PointerState::new(DirPointer::Down, CodelChooser::Left),
    PointerState::new(DirPointer::Down, CodelChooser::Right),
    PointerState::new(DirPointer::Left, CodelChooser::Left),
    PointerState::new(DirPointer::Left, CodelChooser::Right),
    PointerState::new(DirPointer::Up, CodelChooser::Left),
    PointerState::new(DirPointer::Up, CodelChooser::Right),
];

pub trait DirectionOps {
    fn from_idx(i: i64) -> Self;
}

impl DirectionOps for DirPointer {
    fn from_idx(i: i64) -> Self {
        match i {
            0 => DirPointer::Right,
            1 => DirPointer::Down,
            2 => DirPointer::Left,
            3 => DirPointer::Up,
            i => <DirPointer as DirectionOps>::from_idx(i.rem_euclid(4)),
        }
    }
}

impl DirectionOps for CodelChooser {
    fn from_idx(i: i64) -> Self {
        match i {
            0 => CodelChooser::Left,
            1 => CodelChooser::Right,
            i => <CodelChooser as DirectionOps>::from_idx(i.rem_euclid(2)),
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
