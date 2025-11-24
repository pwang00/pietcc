use std::error::Error;

use crate::error::CompilerError;

use crate::{analysis_cache::AnalysisCache, pass::Pass, result::ExecutionResult};
use interpreter::interpreter::Interpreter as StaticEvaluator;
use piet_core::settings::{InterpreterSettings as StaticEvaluatorSettings};
use piet_core::{cfg::CFG, state::ExecutionStatus};

#[derive(Debug)]
pub struct StaticEvaluatorPass;

impl Pass for StaticEvaluatorPass {
    fn name(&self) -> &'static str {
        "static_eval"
    }

    fn run(&mut self, cfg: &mut CFG, analysis_cache: &mut AnalysisCache) -> Result<(), Box<dyn Error>> {
        let max_steps = 20000;
        let codel_settings = piet_core::settings::CodelSettings::Default;
        let static_eval_settings =
            StaticEvaluatorSettings::abstract_interp(max_steps, codel_settings);
        let mut static_eval = StaticEvaluator::new(cfg, static_eval_settings);
        let execution_state = static_eval.run();
        match execution_state.status {
            ExecutionStatus::Completed => Ok(analysis_cache.update_result(ExecutionResult::Complete(execution_state))),
            ExecutionStatus::MaxSteps => Ok(analysis_cache.update_result(ExecutionResult::Partial(execution_state))),
            ExecutionStatus::NeedsInput => Err(Box::new(CompilerError::NotImplementedError)),
            _ => Ok(())
        }
    }
}
