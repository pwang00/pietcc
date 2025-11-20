use crate::context::OptimizationContext;
use piet_core::cfg::CFG;

pub trait Pass {
    fn name(&self) -> &'static str;
    fn run(&mut self, cfg: CFG, ctx: &mut OptimizationContext);
}
