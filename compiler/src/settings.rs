use inkwell::OptimizationLevel;
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
    pub opt_level: OptimizationLevel,
    pub codel_settings: CodelSettings,
    pub save_options: SaveOptions,
    pub output_fname: &'a str,
    pub show_codel_size: bool,
    pub warn_nt: bool,
}
