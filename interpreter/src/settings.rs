use parser::infer::CodelSettings;

#[derive(Copy, Clone, Default)]
pub struct InterpSettings {
    pub verbosity: Verbosity,
    pub codel_settings: CodelSettings,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum Verbosity {
    Low,
    #[default]
    Normal,
    Verbose,
}
