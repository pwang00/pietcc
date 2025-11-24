use inkwell::passes::PassManager;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use parser::cfg::CFGBuilder;
use piet_core::cfg::CFG;
use piet_core::settings::{CompilerSettings, SaveOptions};
use piet_core::state::ExecutionState;
use piet_optimizer::manager::OptimizationPassManager;
use piet_optimizer::result::ExecutionResult;
use piet_optimizer::static_eval::StaticEvaluatorPass;
use std::io::Error;

use crate::builder;
use crate::lowering_ctx::LoweringCtx;
use crate::writer;

fn generate_cfg(cfg_builder: &mut CFGBuilder) {
    cfg_builder.build();
}

pub fn run_pipeline(
    ctx: &mut LoweringCtx,
    mut cfg: &mut CFG,
    settings: CompilerSettings,
) -> Result<(), Error> {
    match settings.opt_level {
        OptimizationLevel::None => builder::build_partial(ctx, cfg, &ExecutionState::default()),
        _ => {
            let mut piet_opt_manager =
                OptimizationPassManager::new(vec![Box::new(StaticEvaluatorPass)], settings.clone());
            piet_opt_manager.run_all(&mut cfg);

            // Build main and constants first as everything will depend on it
            builder::build_main(&ctx);
            builder::build_constants(&ctx);

            if let Some(execution_result) =
                piet_opt_manager.get_analysis_cache().get_cached_result()
            {
                match execution_result {
                    ExecutionResult::Complete(execution_state) => {
                        builder::build_complete(ctx, execution_state)
                    }
                    ExecutionResult::Partial(execution_state) => {
                        builder::build_partial(ctx, &mut cfg, execution_state)
                    }
                }
            }
        }
    }

    match settings.save_options {
        SaveOptions::EmitExecutable => {
            writer::generate_executable(&ctx.module, &settings.output_fname)
        }
        SaveOptions::EmitLLVMBitcode => {
            writer::generate_llvm_bitcode(&ctx.module, &settings.output_fname)
        }
        SaveOptions::EmitLLVMIR => writer::generate_llvm_ir(&ctx.module, &settings.output_fname),
    }
}
