use piet_core::cfg::CFG;

use crate::pass::Pass;

pub struct OptimizationContext {
    cfg: CFG,
    result: ExecutionResult,
}
