use crate::builder;
use crate::llvm_pipeline::run_llvm_optimizations;
use crate::lowering_ctx::LoweringCtx;
use crate::writer;
use inkwell::passes::{PassBuilderOptions, PassManager};
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use parser::cfg::CFGBuilder;
use piet_core::cfg::CFG;
use piet_core::settings::{CompilerSettings, SaveOptions};
use piet_core::state::ExecutionState;
use piet_optimizer::manager::OptimizationPassManager;
use piet_optimizer::result::ExecutionResult;
use piet_optimizer::static_eval::StaticEvaluatorPass;
use std::error::Error;

fn generate_cfg(cfg_builder: &mut CFGBuilder) {
    cfg_builder.build();
}

pub fn run_pipeline(
    ctx: &mut LoweringCtx,
    cfg: &mut CFG,
    settings: CompilerSettings,
) -> Result<(), Box<dyn Error>> {
    // Build globals - declares all functions and global variables
    builder::build_globals(ctx);

    match settings.opt_level {
        OptimizationLevel::None => builder::build_partial(ctx, cfg, &ExecutionState::default()),
        _ => {
            let mut piet_opt_manager =
                OptimizationPassManager::new(vec![Box::new(StaticEvaluatorPass)], settings);
            piet_opt_manager.run_all(cfg);
            if let Some(execution_result) =
                piet_opt_manager.get_analysis_cache().get_cached_result()
            {
                match execution_result {
                    ExecutionResult::Complete(execution_state) => {
                        builder::build_complete(ctx, execution_state)
                    }
                    ExecutionResult::Partial(execution_state) => {
                        builder::build_partial(ctx, cfg, execution_state)
                    }
                }
            }
        }
    }
    run_llvm_optimizations(ctx)?;
    match settings.save_options {
        SaveOptions::EmitExecutable => Ok(writer::generate_executable(
            &ctx.module,
            &settings.output_fname,
        )?),
        SaveOptions::EmitLLVMBitcode => Ok(writer::generate_llvm_bitcode(
            &ctx.module,
            &settings.output_fname,
        )?),
        SaveOptions::EmitLLVMIR => Ok(writer::generate_llvm_ir(
            &ctx.module,
            &settings.output_fname,
        )?),
    }
}
