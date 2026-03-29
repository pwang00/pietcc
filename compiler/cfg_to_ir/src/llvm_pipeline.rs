use std::error::Error;

use crate::lowering_ctx::LoweringCtx;
use inkwell::targets::{InitializationConfig, RelocMode, Target};
use inkwell::OptimizationLevel;
use inkwell::{passes::PassBuilderOptions, targets::TargetMachine};
use piet_optimizer::error::OptimizerError;

pub(crate) fn run_llvm_optimizations(ctx: &LoweringCtx) -> Result<(), Box<dyn Error>> {
    let default_triple = TargetMachine::get_default_triple();
    let options = PassBuilderOptions::create();
    let opt_level = match ctx.settings.opt_level {
        OptimizationLevel::None => None,
        OptimizationLevel::Less => Some("default<O1>"),
        OptimizationLevel::Default => Some("default<O2>"),
        OptimizationLevel::Aggressive => Some("default<O3>"),
    };

    if let Some(opt_level) = opt_level {
        Target::initialize_native(&InitializationConfig::default())?;
        let target = Target::from_triple(&default_triple).expect("No target for triple");
        let tm = target
            .create_target_machine(
                &default_triple,
                "",
                "",
                ctx.settings.opt_level,
                RelocMode::PIC,
                inkwell::targets::CodeModel::Default,
            )
            .expect("Failed to create TargetMachine");

        Ok(ctx
            .module
            .run_passes(opt_level, &tm, options)
            .map_err(|llvm_string| Box::new(OptimizerError::LLVMError(llvm_string.to_string())))?)
    } else {
        Ok(())
    }
}
