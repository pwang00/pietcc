use inkwell::OptimizationLevel;
use interpreter::settings::PartialEvalSettings;
use parser::infer::CodelSettings;

#[derive(Copy, Clone, Default, Debug)]
pub enum SaveOptions {
    #[default]
    EmitExecutable,
    EmitLLVMBitcode,
    EmitLLVMIR,
}

#[derive(Copy, Clone, Debug)]
pub struct CompilerSettings<'a> {
    pub llvm_opt_level: OptimizationLevel,
    pub partial_eval_settings: Option<PartialEvalSettings>,
    pub codel_settings: CodelSettings,
    pub save_options: SaveOptions,
    pub output_fname: &'a str,
    pub show_codel_size: bool,
    pub show_cfg_size: bool,
    pub warn_nt: bool,
}
