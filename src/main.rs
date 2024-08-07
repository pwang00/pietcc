pub mod verbosity;

use crate::Verbosity;
use clap::{App, Arg};
use compiler::codegen::CodeGen;
use compiler::settings::CompilerSettings;
use compiler::{cfg_gen::CFGGenerator, settings::SaveOptions};
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use interpreter::{interpreter::Interpreter, settings::*};
use parser::convert::UnknownPixelSettings;
use parser::{infer::CodelSettings, loader::Loader};
use std::env;
use std::io::Error;
use std::process::exit;
use types::program::Program;

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
                .help("Sets the compiler optimization level to 1 (LLVM Less)"),
        )
        .arg(
            Arg::with_name("o2")
                .long("o2")
                .takes_value(false)
                .conflicts_with("o1")
                .conflicts_with("interpret")
                .conflicts_with("o3")
                .help("Sets the compiler optimization level to 2 (LLVM Default)"),
        )
        .arg(
            Arg::with_name("o3")
                .long("o3")
                .takes_value(false)
                .conflicts_with("o1")
                .conflicts_with("interpret")
                .conflicts_with("o2")
                .help("Sets the compiler optimization level to 3 (LLVM Aggressive)"),
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
        .get_matches();

    let filename = matches.value_of("input").unwrap();
    let mut interpreter: Interpreter;
    let program: Program;
    let mut behavior = UnknownPixelSettings::TreatAsError;

    if matches.is_present("treat_white") {
        behavior = UnknownPixelSettings::TreatAsWhite
    }

    if matches.is_present("treat_black") {
        behavior = UnknownPixelSettings::TreatAsBlack
    }

    let res = Loader::convert(filename, behavior);

    if let Ok(prog) = res {
        program = prog;
        let mut codel_settings = CodelSettings::Infer;
        let mut verbosity = Verbosity::Normal;
        let mut interp_settings = InterpSettings {
            verbosity: Verbosity::Normal,
            codel_settings,
        };

        if let Some(val) = matches.value_of("codel_size") {
            if let Ok(val) = val.parse::<u32>() {
                if program.dimensions().0 % val != 0 || program.dimensions().1 % val != 0 {
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

        if let Some(_) = matches.value_of("use_default") {
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

        if matches.is_present("interpret") {
            interp_settings.codel_settings = codel_settings;
            interpreter = Interpreter::new(&program, interp_settings);
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

            let show_codel_size = match verbosity {
                Verbosity::Low | Verbosity::Normal => false,
                _ => true,
            };

            let show_cfg_size = match verbosity {
                Verbosity::Low => false,
                _ => true,
            };

            let compile_options = CompilerSettings {
                opt_level,
                codel_settings,
                save_options,
                output_fname,
                warn_nt,
                show_cfg_size,
                show_codel_size,
            };

            let cfg_gen = CFGGenerator::new(&program, codel_settings, show_codel_size);
            let mut cg = CodeGen::new(&context, module, builder, cfg_gen, compile_options);
            if let Err(e) = cg.run(compile_options) {
                println!("{:?}", e);
            }
        }
    } else {
        match env::consts::OS {
            "linux" => {
                eprintln!("\x1B[1;37mpietcc: \x1B[0m\x1B[1;31mfatal error: \x1B[0m{}: No such file or directory.", filename);
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
