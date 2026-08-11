pub mod loader;
pub mod verbosity;

use crate::Verbosity;
use crate::loader::to_lightness_raster;
use cfg_to_ir::lowering_ctx::LoweringCtx;
use cfg_to_ir::pipeline;
use clap::{App, Arg};
use frontend::conversions::UnknownPixelSettings;
use frontend::pipeline::run_frontend_pipeline;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use interpreter::interpreter::Interpreter;
use piet_core::settings::*;
use std::env;
use std::io::Error;
use std::process::exit;

fn main() -> Result<(), Error> {
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
                .default_value("program.out")
                .help("Output an executable into <file>"),
        )
        .arg(
            Arg::with_name("codel_size")
                .short('s')
                .long("size")
                .takes_value(true)
                .help("Interpret or compile with a supplied codel size (must divide program height and width)"),
        )
        .arg(
            Arg::with_name("use_default")
                .short('d')
                .long("default")
                .takes_value(true)
                .help("Interpret or compile with a codel size of 1"),
        )
        .arg(
            Arg::with_name("emit-llvm")
                .long("emit-llvm")
                .takes_value(false)
                .help("Emit LLVM IR for a given Piet program"),
        )
        .arg(
            Arg::with_name("emit-llvm-bitcode")
                .long("emit-llvm-bitcode")
                .takes_value(false)
                .conflicts_with("emit-llvm")
                .help("Emit LLVM bitcode for a given Piet program"),
        )
        .arg(
            Arg::with_name("verbosity")
                .short('v')
                .long("verbosity")
                .takes_value(true)
                .default_missing_value("0")
                .help("Sets the interpreter or compiler's verbosity"),
        )
        .arg(
            Arg::with_name("o1")
                .long("o1")
                .takes_value(false)
                .conflicts_with("o2")
                .conflicts_with("interpret")
                .conflicts_with("o3")
                .help("Sets the compiler optimization level to 1 (LLVM default<O1>, attempts Piet compile-time evaluation to fold constant programs)"),
        )
        .arg(
            Arg::with_name("o2")
                .long("o2")
                .takes_value(false)
                .conflicts_with("o1")
                .conflicts_with("interpret")
                .conflicts_with("o3")
                .help("Sets the compiler optimization level to 2 (LLVM default<O2>, attempts Piet compile-time evaluation to fold constant programs)"),
        )
        .arg(
            Arg::with_name("o3")
                .long("o3")
                .takes_value(false)
                .conflicts_with("o1")
                .conflicts_with("interpret")
                .conflicts_with("o2")
                .help("Sets the compiler optimization level to 3 (LLVM default<O3>, attempts Piet compile-time evaluation to fold constant programs)"),
        )
        .arg(
            Arg::with_name("treat_white")
                .long("uw")
                .takes_value(false)
                .conflicts_with("treat_black")
                .help("Treats unknown pixels as white (default: error)"),
        )
        .arg(
            Arg::with_name("treat_black")
                .long("ub")
                .takes_value(false)
                .conflicts_with("treat_white")
                .help("Treats unknown pixels as black (default: error)"),
        )
        .arg(
            Arg::with_name("warn_nontermination")
                .short('w')
                .long("warn-nt")
                .takes_value(false)
                .requires("out")
                .conflicts_with("interpret")
                .help("Attempts to detect nontermination behavior in a Piet program during compilation"),
        )
        .arg(
            Arg::with_name("max_steps")
                .short('m')
                .long("max-steps")
                .takes_value(true)
                .help("Sets the maximum number of steps for the interpreter to execute"),
        )
        .get_matches();

    let filename = matches.value_of("input").unwrap();
    let mut interpreter: Interpreter;
    let mut unknown_pixel_behavior = UnknownPixelSettings::TreatAsError;

    if matches.is_present("treat_white") {
        unknown_pixel_behavior = UnknownPixelSettings::TreatAsWhite
    }

    if matches.is_present("treat_black") {
        unknown_pixel_behavior = UnknownPixelSettings::TreatAsBlack
    }

    let res = to_lightness_raster(filename, unknown_pixel_behavior);

    if let Ok(program) = res {
        let mut codel_settings = CodelSettings::Infer;
        let mut verbosity = Verbosity::Normal;
        let mut interp_settings = InterpreterSettings::default();

        if let Some(val) = matches.value_of("codel_size") {
            if let Ok(val) = val.parse::<usize>() {
                if !program.dimensions().0.is_multiple_of(val)
                    || !program.dimensions().1.is_multiple_of(val)
                {
                    match env::consts::OS {
                        "linux" => {
                            eprintln!(
                                "\x1B[1;37mpietcc: \x1B[0m\x1B[1;31mfatal error: \x1B[0m{}: supplied codel width {} does not divide program dimensions: {:?}",
                                filename,
                                val,
                                program.dimensions()
                            );
                            eprintln!("pietcc terminated.");
                        }
                        _ => {
                            eprintln!(
                                "pietcc: fatal error: {}: supplied codel width {} does not divide program dimensions: {:?}",
                                filename,
                                val,
                                program.dimensions()
                            );
                            eprintln!("pietcc terminated.");
                        }
                    }
                    exit(1);
                }

                codel_settings = CodelSettings::Width(val);
            }
        }

        if matches.value_of("use_default").is_some() {
            codel_settings = CodelSettings::Default
        }

        if let Some(val) = matches.value_of("verbosity") {
            verbosity = match val {
                "0" => Verbosity::Low,
                "2" => Verbosity::Verbose,
                _ => Verbosity::Normal,
            };
            interp_settings.verbosity = verbosity;
        }

        if let Some(val) = matches.value_of("max_steps") {
            if let Ok(steps) = val.parse::<u64>() {
                interp_settings.max_steps = Some(steps);
            } else {
                match env::consts::OS {
                    "linux" => {
                        eprintln!(
                            "\x1B[1;37mpietcc: \x1B[0m\x1B[1;31mfatal error: \x1B[0minvalid value for max-steps: {}",
                            val
                        );
                        eprintln!("pietcc terminated.");
                    }
                    _ => {
                        eprintln!("pietcc: fatal error: invalid value for max-steps: {}", val);
                        eprintln!("pietcc terminated.");
                    }
                }
                exit(1);
            }
        }

        let mut cfg = run_frontend_pipeline(program, codel_settings);

        if matches.is_present("interpret") {
            interp_settings.codel_settings = codel_settings;
            interpreter = Interpreter::new(&cfg, interp_settings);
            println!("\n{}", interpreter.run());
            exit(0);
        }

        if let Some(output_fname) = matches.value_of("out") {
            let context = Context::create();
            let module = context.create_module("piet");
            let builder = context.create_builder();
            // Program

            let mut save_options = SaveOptions::EmitExecutable;
            let mut opt_level = OptimizationLevel::None;

            if matches.is_present("emit-llvm") {
                save_options = SaveOptions::EmitLLVMIR
            } else if matches.is_present("emit-llvm-bitcode") {
                save_options = SaveOptions::EmitLLVMBitcode
            }

            if matches.is_present("o1") {
                opt_level = OptimizationLevel::Less
            } else if matches.is_present("o2") {
                opt_level = OptimizationLevel::Default
            } else if matches.is_present("o3") {
                opt_level = OptimizationLevel::Aggressive
            }

            let warn_nt = matches.is_present("warn_nontermination");
            let show_codel_size = !matches!(verbosity, Verbosity::Low | Verbosity::Normal);
            let show_cfg_size = !matches!(verbosity, Verbosity::Low);

            let compile_options = CompilerSettings {
                opt_level,
                codel_settings,
                save_options,
                output_fname,
                warn_nt,
                show_cfg_size,
                show_codel_size,
                verbosity,
            };

            let mut piet_ctx = LoweringCtx::new(&context, module, builder, compile_options);
            if let Err(e) =
                pipeline::run_piet_optimization_pipeline(&mut piet_ctx, &mut cfg, compile_options)
            {
                println!("{:?}", e);
            }
        }
    } else {
        match env::consts::OS {
            "linux" => {
                eprintln!(
                    "\x1B[1;37mpietcc: \x1B[0m\x1B[1;31mfatal error: \x1B[0m{}: No such file or directory.",
                    filename
                );
                eprintln!("pietcc terminated.");
            }
            _ => {
                eprintln!(
                    "pietcc: fatal error: {}: No such file or directory.",
                    filename
                );
                eprintln!("pietcc terminated.");
            }
        }
        exit(1);
    }
    Ok(())
}
