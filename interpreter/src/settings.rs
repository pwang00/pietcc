use parser::infer::CodelSettings;

#[derive(Copy, Clone, Default)]
pub struct InterpSettings {
    pub verbosity: Verbosity,
    pub codel_settings: CodelSettings,
    pub partial_eval_settings: Option<PartialEvalSettings>
}

impl InterpSettings {
    pub fn partial_evaluation(max_steps: u32, codel_settings: CodelSettings) -> Self {
        InterpSettings {
            verbosity: Verbosity::Low,
            codel_settings,
            max_steps: Some(max_steps),
            collect_instructions_only: true
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
