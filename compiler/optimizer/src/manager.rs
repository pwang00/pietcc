use crate::analysis_cache;
use crate::{analysis_cache::AnalysisCache, pass::Pass};
use piet_core::cfg::CFG;
use piet_core::settings::CompilerSettings;

#[derive(Debug)]
pub struct OptimizationPassManager {
    passes: Vec<Box<dyn Pass>>,
    analysis_cache: AnalysisCache,
    settings: CompilerSettings,
}

impl OptimizationPassManager {
    pub fn run_all(&mut self, cfg: &mut CFG) {
        // Borrow checker :(
        for pass in &mut self.passes {
            pass.run(cfg, &mut self.analysis_cache);
        }
    }

    pub fn new(passes: Vec<Box<dyn Pass>>, settings: CompilerSettings) -> Self {
        Self {
            passes,
            analysis_cache: AnalysisCache::default(),
            settings,
        }
    }

    pub fn get_analysis_cache(&self) -> &AnalysisCache {
        &self.analysis_cache
    }
}
