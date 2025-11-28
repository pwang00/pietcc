use std::collections::VecDeque;

use crate::{flow::*, instruction::*};
pub type Position = (u32, u32);

pub const ENTRY: Position = (0, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionStatus {
    #[default]
    Running,
    Completed,
    MaxSteps,
    NeedsInput,
}

/// Immmediate state information
#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub pointers: PointerState,
    pub cb_count: u64,
    pub cb_label: String,
    pub stdin: String,
    pub stdout: Vec<StdOutWrapper>,
    pub steps: u64,
    pub status: ExecutionStatus, // If program ran to completion vs just hitting max steps
    pub stack: VecDeque<i64>,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            pointers: Default::default(),
            cb_count: Default::default(),
            cb_label: "Entry".into(),
            stdin: Default::default(),
            stdout: Default::default(),
            steps: Default::default(),
            status: Default::default(),
            stack: Default::default(),
        }
    }
}
#[allow(unused_must_use)]
impl std::fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ExecutionState {{");
        writeln!(f, "    dp: {:?}", self.pointers.dp);
        writeln!(f, "    cc: {:?}", self.pointers.cc);
        writeln!(f, "    cb: {:?}", self.cb_count);
        writeln!(f, "    steps: {:?}", self.steps);
        writeln!(f, "    status: {:?}", self.status);
        writeln!(f, "    stack: {:?}", self.stack);
        writeln!(f, "}}")
    }
}
