use std::collections::HashSet;

use crate::{instruction::Instruction, program::PietSource, state::Position};

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

pub fn find_offset(curr: PointerState, target: PointerState) -> u8 {
    let mut state = curr;

    for attempts in 0..8 {
        if state.dp == target.dp && state.cc == target.cc {
            return attempts;
        }

        if attempts % 2 == 0 {
            state.cc = match state.cc {
                CodelChooser::Left => CodelChooser::Right,
                CodelChooser::Right => CodelChooser::Left,
            };
        } else {
            // Rotate dp: 0 -> 1 -> 2 -> 3 -> 0
            state.dp = match state.dp {
                DirPointer::Right => DirPointer::Down,
                DirPointer::Down => DirPointer::Left,
                DirPointer::Left => DirPointer::Up,
                DirPointer::Up => DirPointer::Right,
            };
        }
    }

    unreachable!()
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
#[derive(Debug, Copy, Clone)]
pub struct PietTransition {
    pub entry_state: PointerState,
    pub exit_state: PointerState,
    pub instruction: Option<Instruction>,
}

impl PietTransition {
    pub fn new(
        entry_state: PointerState,
        exit_state: PointerState,
        instruction: Option<Instruction>,
    ) -> Self {
        Self {
            entry_state,
            exit_state,
            instruction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_offset_retry_sequence() {
        let right_left = PointerState::new(DirPointer::Right, CodelChooser::Left);
        let right_right = PointerState::new(DirPointer::Right, CodelChooser::Right);
        let down_left = PointerState::new(DirPointer::Down, CodelChooser::Left);
        let down_right = PointerState::new(DirPointer::Down, CodelChooser::Right);

        // Retry simulation: toggle cc (even), rotate dp (odd)
        assert_eq!(find_offset(right_left, right_right), 1);
        assert_eq!(find_offset(right_left, down_right), 2);
        assert_eq!(find_offset(right_left, down_left), 3);
        assert_eq!(find_offset(down_left, right_left), 7);
    }
}
