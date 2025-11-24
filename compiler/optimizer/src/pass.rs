use crate::analysis_cache::AnalysisCache;
use piet_core::cfg::CFG;
use std::{error::Error, fmt::Debug};

pub trait Pass: Debug {
    fn name(&self) -> &'static str;
    fn run(&mut self, cfg: &mut CFG, manager: &mut AnalysisCache) -> Result<(), Box<dyn Error>>;
}
