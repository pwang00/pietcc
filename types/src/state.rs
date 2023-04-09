use std::collections::VecDeque;

use crate::flow::*;
pub type Position = (u32, u32);

pub const ENTRY: Position = (0, 0);
#[derive(Debug, Default)]
pub struct ExecutionState {
    pub dp: Direction,
    pub cc: Codel,
    pub pos: Position,
    pub cb: u64,
    pub rctr: u8,
    pub stdin: String,
    pub steps: u64,
}

pub struct ExecutionResult<'a> {
    pub state: &'a ExecutionState,
    pub stack: &'a VecDeque<i64>,
}

#[allow(unused_must_use)]
impl std::fmt::Display for ExecutionResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExecutionResult {{");
        writeln!(f, "    dp: {:?}", self.state.dp);
        writeln!(f, "    cc: {:?}", self.state.cc);
        writeln!(f, "    pos: {:?}", self.state.pos);
        writeln!(f, "    cb: {:?}", self.state.cb);
        writeln!(f, "    rctr: {:?}", self.state.rctr);
        writeln!(f, "    stdin: {:?}", self.state.stdin);
        writeln!(f, "    steps: {:?}", self.state.steps);
        writeln!(f, "    stack: {:?}", self.stack);
        writeln!(f, "}}")
    }
}
