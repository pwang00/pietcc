use piet_core::state::ExecutionState;
use piet_optimizer::result::ExecutionResult;

use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_from_state(&self, execution_result: ExecutionResult) {
        // If execution result is complete, then simply compile stdout
        // Otherwise, we need to build the stack, determine the correct color block to jump to, and set dp / cc correctly
        match execution_result {
            ExecutionResult::Complete(execution_state) => self.build_complete(execution_state),
            ExecutionResult::Partial(execution_state) => self.build_partial(execution_state),
        }
    }

    fn build_complete(&self, execution_state: ExecutionState) {
        let stack = execution_state.stack;
    }

    fn build_partial(&self, execution_state: ExecutionState) {}
}
