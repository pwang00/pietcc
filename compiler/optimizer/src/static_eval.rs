use crate::{analysis_cache::AnalysisCache, pass::Pass, result::ExecutionResult};
use interpreter::interpreter::Interpreter as StaticEvaluator;
use piet_core::settings::InterpreterSettings as StaticEvaluatorSettings;
use piet_core::{cfg::CFG, state::ExecutionStatus};

#[derive(Debug)]
pub struct StaticEvaluatorPass;

impl Pass for StaticEvaluatorPass {
    fn name(&self) -> &'static str {
        "static_eval"
    }

    fn run(&mut self, cfg: &mut CFG, analysis_cache: &mut AnalysisCache) {
        let max_steps = 2000;
        let codel_settings = piet_core::settings::CodelSettings::Default;
        let static_eval_settings =
            StaticEvaluatorSettings::abstract_interp(max_steps, codel_settings, false);
        let mut static_eval = StaticEvaluator::new(cfg, static_eval_settings);
        let execution_state = static_eval.run();
        match execution_state.status {
            ExecutionStatus::Completed => {
                analysis_cache.update_result(ExecutionResult::Complete(execution_state))
            }
            ExecutionStatus::MaxSteps | ExecutionStatus::NeedsInput => todo!(),
            _ => analysis_cache.update_result(ExecutionResult::Partial(execution_state)),
        }
    }
}
