use types::flow::{Codel, DirVec, Direction};

pub const STACK_SIZE: u32 = 1 << 18;

// Globals
pub const PIET_STACK: &str = "piet_stack";

// Functions
pub const RETRY_FN: &str = "retry";
pub const MAIN: &str = "main";

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

