use std::io::Error;

use clap::{App, Arg};
use compiler::codegen::CodeGen;
use compiler::settings::CompilerSettings;
use compiler::{cfg_gen::CFGGenerator, settings::SaveOptions};
use inkwell::context::Context;
use interpreter::{interpreter::Interpreter, settings::*};
use parser::{infer::CodelSettings, loader::Loader};
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
                .requires("interpret")
                .help("Sets the interpreter's verbosity"),
        )
        .get_matches();

    let filename = matches.value_of("input").unwrap();
    let mut interpreter: Interpreter;
    let program: Program;

    if let Ok(prog) = Loader::convert(filename) {
        program = prog;
        let mut codel_settings = CodelSettings::Infer;

        let mut interp_settings = InterpSettings {
            verbosity: Verbosity::Normal,
            codel_settings,
        };

        if let Some(val) = matches.value_of("codel_size") {
            if let Ok(val) = val.parse::<u32>() {
                codel_settings = CodelSettings::Width(val);
            }
        }

        if let Some(_) = matches.value_of("use_default") {
            codel_settings = CodelSettings::Default
        }

        if let Some(val) = matches.value_of("verbosity") {
            let verbosity = match val {
                "0" => Verbosity::Low,
                "2" => Verbosity::Verbose,
                _ => Verbosity::Normal,
            };
            interp_settings.verbosity = verbosity;

            println!("Running with verbosity set to {:?}", verbosity);
        }

        if matches.is_present("interpret") {
            interp_settings.codel_settings = codel_settings;
            interpreter = Interpreter::new(&program, interp_settings);
            println!("\n{}", interpreter.run());
        }

        if let Some(output_fname) = matches.value_of("out") {
            let context = Context::create();
            let module = context.create_module("piet");
            let builder = context.create_builder();
            // Program

            let mut save_options = SaveOptions::EmitExecutable;

            if matches.is_present("emit-llvm") {
                save_options = SaveOptions::EmitLLVMIR
            }

            if matches.is_present("emit-llvm-bitcode") {
                save_options = SaveOptions::EmitLLVMBitcode
            }

            let compile_options = CompilerSettings {
                opt_level: inkwell::OptimizationLevel::None,
                codel_settings,
                save_options,
                output_fname,
            };

            let cfg_gen = CFGGenerator::new(&program, codel_settings);
            let mut cg = CodeGen::new(&context, module, builder, cfg_gen, compile_options);
            if let Err(e) = cg.run(compile_options) {
                println!("{:?}", e);
            }
        }
    }
    Ok(())
}
