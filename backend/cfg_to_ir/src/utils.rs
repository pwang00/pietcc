use piet_core::settings::Verbosity;

pub(crate) fn vprint(verbosity: Verbosity, msg: &str) {
    match verbosity {
        Verbosity::Low => (),
        Verbosity::Normal | Verbosity::Verbose => {
            println!("{msg}");
        }
    }
}
