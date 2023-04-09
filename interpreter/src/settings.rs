#[derive(Copy, Clone)]
pub struct InterpSettings {
    pub verbosity: Verbosity,
    pub codel_settings: CodelSettings,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Verbosity {
    Low,
    Normal,
    Verbose,
}

#[derive(Copy, Clone)]
pub enum CodelSettings {
    Default,
    Infer,
    Width(u32),
}
