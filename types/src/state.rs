use std::{collections::VecDeque, fmt::Pointer};

use crate::{flow::*, instruction::*};
pub type Position = (u32, u32);

pub const ENTRY: Position = (0, 0);

/// Immmediate state information excluding stack
#[derive(Debug, Default, Clone)]
pub struct ExecutionState {
    pub pointers: PointerState,
    pub cb: u64,
    pub stdin: String,
    pub stdout: Vec<StdOutWrapper>,
    pub steps: u64,
    pub complete: bool, // If program ran to completion vs just hitting max steps
    pub stack: VecDeque<i64>,
}
#[allow(unused_must_use)]
impl std::fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExecutionState {{");
        writeln!(f, "    dp: {:?}", self.pointers.dp);
        writeln!(f, "    cc: {:?}", self.pointers.cc);
        writeln!(f, "    cb: {:?}", self.cb);
        writeln!(f, "    stdin: {:?}", self.stdin);
        writeln!(f, "    stdout: {:?}", self.stdout);
        writeln!(f, "    steps: {:?}", self.steps);
        writeln!(f, "    stack: {:?}", self.stack);
        writeln!(f, "}}")
    }
}
