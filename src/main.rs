use clap::{App, Arg};
use interpreter::{interpreter::Interpreter, settings::*};
use parser::loader::Loader;
use types::program::Program;

fn main() {
    let matches = App::new("pietcc")
        .about("Piet compiler and interpreter")
        .arg(
            Arg::with_name("input")
                .required(true)
                .takes_value(true)
                .help("Piet source file to interpret")
                .index(1),
        )
        .arg(
            Arg::with_name("interpret")
                .short('i')
                .long("interpret")
                .required(false)
                .takes_value(false)
                .help("Interpret the given program"),
        )
        .arg(
            Arg::with_name("out")
                .short('o')
                .long("output")
                .takes_value(true)
                .required_unless_present("interpret")
                .help("Place the output into <file>"),
        )
        .arg(
            Arg::with_name("codel_size")
                .short('s')
                .long("size")
                .takes_value(true)
                .help("Interpret or compile with a supplied codel size"),
        )
        .arg(
            Arg::with_name("use_default")
                .short('d')
                .long("default")
                .takes_value(true)
                .help("Interpret or compile with a codel size of 1"),
        )
        .arg(
            Arg::with_name("verbosity")
                .short('v')
                .long("verbosity")
                .takes_value(true)
                .default_missing_value("0")
                .requires("interpret")
                .help("Sets the interpreter's verbosity"),
        )
        .get_matches();

    let filename = matches.value_of("input").unwrap();
    let mut interpreter: Interpreter;
    let program: Program;

    if let Ok(prog) = Loader::convert(filename) {
        program = prog;

        let mut settings = InterpSettings {
            verbosity: Verbosity::Normal,
            codel_settings: CodelSettings::Infer,
        };

        if let Some(val) = matches.value_of("codel_size") {
            if let Ok(val) = val.parse::<u32>() {
                settings.codel_settings = CodelSettings::Width(val);
            }
        }

        if let Some(_) = matches.value_of("use_default") {
            settings.codel_settings = CodelSettings::Default
        }

        if let Some(val) = matches.value_of("verbosity") {
            let verbosity = match val {
                "0" => Verbosity::Low,
                "2" => Verbosity::Verbose,
                _ => Verbosity::Normal,
            };
            settings.codel_settings = CodelSettings::Default;
            settings.verbosity = verbosity;

            println!("Running with verbosity set to {:?}", verbosity);
        }

        if matches.is_present("interpret") {
            interpreter = Interpreter::new(&program, settings);
            println!("\n{}", interpreter.run());
        }
    }
}
