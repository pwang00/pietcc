use piet_core::state::ExecutionState;

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Complete(ExecutionState),
    Partial(ExecutionState),
}
