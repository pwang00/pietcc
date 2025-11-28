use parser::cfg::CFGBuilder;
use piet_core::settings::Verbosity;

#[allow(unused)]
pub(crate) fn generate_cfg(cfg_builder: &mut CFGBuilder) {
    cfg_builder.build();
}

pub(crate) fn vprint(verbosity: Verbosity, msg: &str) {
    match verbosity {
        Verbosity::Low => (),
        Verbosity::Normal | Verbosity::Verbose => {
            println!("{msg}");
        }
    }
}
