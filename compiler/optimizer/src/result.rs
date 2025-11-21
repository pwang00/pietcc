use piet_core::{cfg::CFG, state::ExecutionState};

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Complete(ExecutionState),
    Partial(ExecutionState),
}
