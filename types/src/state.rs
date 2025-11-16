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
    pub steps: u64,
}

pub struct ExecutionResult<'a> {
    pub state: &'a ExecutionState,
    pub stack: &'a VecDeque<i64>,
    pub stdout: &'a Vec<(Instruction, i64)>,
}

#[allow(unused_must_use)]
impl std::fmt::Display for ExecutionResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExecutionResult {{");
        writeln!(f, "    dp: {:?}", self.state.pointers.dp);
        writeln!(f, "    cc: {:?}", self.state.pointers.cc);
        writeln!(f, "    cb: {:?}", self.state.cb);
        writeln!(f, "    stdin: {:?}", self.state.stdin);
        writeln!(f, "    steps: {:?}", self.state.steps);
        writeln!(f, "    stack: {:?}", self.stack);
        writeln!(f, "}}")
    }
}
