#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum Verbosity {
    Low,
    #[default]
    Normal,
    Verbose,
}
