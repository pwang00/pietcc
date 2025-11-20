use parser::infer::CodelSettings;

#[derive(Copy, Clone, Default)]
pub struct InterpSettings {
    pub verbosity: Verbosity,
    pub codel_settings: CodelSettings,
    pub max_steps: Option<u64>,
    pub partial_eval: bool,
    pub abstract_interp: bool,
    pub print: bool,
}

impl InterpSettings {
    pub fn partial_evaluation(max_steps: u64, codel_settings: CodelSettings, print: bool) -> Self {
        InterpSettings {
            verbosity: Verbosity::Low,
            codel_settings,
            max_steps: Some(max_steps),
            partial_eval: true,
            abstract_interp: false,
            print,
        }
    }

    pub fn abstract_interp(max_steps: u64, codel_settings: CodelSettings, print: bool) -> Self {
        InterpSettings {
            verbosity: Verbosity::Low,
            codel_settings,
            max_steps: Some(max_steps),
            partial_eval: true,
            abstract_interp: true,
            print,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PartialEvalSettings {
    pub max_steps: u64,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum Verbosity {
    Low,
    #[default]
    Normal,
    Verbose,
}
