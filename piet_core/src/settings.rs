use inkwell::OptimizationLevel;

#[derive(Copy, Clone, Default, Debug)]
pub enum CodelSettings {
    #[default]
    Default,
    Infer,
    Width(u32),
}

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
    pub show_cfg_size: bool,
    pub warn_nt: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct InterpreterSettings {
    pub verbosity: Verbosity,
    pub codel_settings: CodelSettings,
    pub max_steps: Option<u64>,
    pub partial_eval: bool,
    pub abstract_interp: bool,
    pub print: bool,
}

impl Default for InterpreterSettings {
    fn default() -> Self {
        Self { 
            verbosity: Default::default(), 
            codel_settings: Default::default(), 
            max_steps: Default::default(), 
            partial_eval: Default::default(), 
            abstract_interp: Default::default(), 
            print: true
        }
    }
}

impl InterpreterSettings {
    pub fn abstract_interp(max_steps: u64, codel_settings: CodelSettings) -> Self {
        InterpreterSettings {
            verbosity: Verbosity::Low,
            codel_settings,
            max_steps: Some(max_steps),
            partial_eval: true,
            abstract_interp: true,
            print: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum Verbosity {
    Low,
    #[default]
    Normal,
    Verbose,
}
