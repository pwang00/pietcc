use std::error::Error;

use crate::error::OptimizerError;

use crate::{analysis_cache::AnalysisCache, pass::Pass, result::ExecutionResult};
use interpreter::interpreter::Interpreter as StaticEvaluator;
use piet_core::settings::InterpreterSettings as StaticEvaluatorSettings;
use piet_core::{cfg::CFG, state::ExecutionStatus};

#[derive(Debug)]
pub struct StaticEvaluatorPass;

pub const MAX_STEPS: u64 = 200000;

impl Pass for StaticEvaluatorPass {
    fn name(&self) -> &'static str {
        "static_eval"
    }

    fn run(
        &mut self,
        cfg: &mut CFG,
        analysis_cache: &mut AnalysisCache,
    ) -> Result<(), Box<dyn Error>> {
        let codel_settings = piet_core::settings::CodelSettings::Default;
        let static_eval_settings =
            StaticEvaluatorSettings::abstract_interp(MAX_STEPS, codel_settings);
        let mut static_eval = StaticEvaluator::new(cfg, static_eval_settings);
        let execution_state = static_eval.run();
        match execution_state.status {
            ExecutionStatus::Completed => {
                Ok(analysis_cache.update_result(ExecutionResult::Complete(execution_state)))
            }
            ExecutionStatus::MaxSteps => {
                Ok(analysis_cache.update_result(ExecutionResult::Partial(execution_state)))
            }
            ExecutionStatus::NeedsInput => Err(Box::new(OptimizerError::StaticEvaluationError(
                "Static evaluation on input-dependent programs not yet implemented.  Will compile with partial execution result.".into(),
                ExecutionResult::Partial(execution_state),
            ))),
            _ => Ok(()),
        }
    }
}
