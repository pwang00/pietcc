use crate::piet_to_cfg::CFGBuilder;
use crate::settings::{CompilerSettings, SaveOptions};
use inkwell::targets::{InitializationConfig, Target};
use inkwell::{builder::Builder, context::Context, module::Module};

use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::OptimizationLevel;
use interpreter::settings::InterpSettings;
use parser::decode::DecodeInstruction;
use types::program;
use std::env;
use std::fs::{remove_file, OpenOptions};
use std::io::{Error, Write};
use std::process::Command;
use types::instruction::Instruction;
use interpreter::interpreter::Interpreter;

#[allow(unused)]
pub struct CodeGen<'a, 'b> {
    pub(crate) context: &'b Context,
    pub(crate) module: Module<'b>,
    pub(crate) builder: Builder<'b>,
    pub(crate) cfg_builder: CFGBuilder<'a>,
    pub(crate) settings: CompilerSettings<'a>,
}

impl<'a, 'b> DecodeInstruction for CodeGen<'a, 'b> {}

#[allow(unused)]
impl<'a, 'b> CodeGen<'a, 'b> {
    pub fn new(
        context: &'b Context,
        module: Module<'b>,
        builder: Builder<'b>,
        cfg_builder: CFGBuilder<'a>,
        settings: CompilerSettings<'a>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            cfg_builder,
            settings,
        }
    }

    fn generate_cfg(&mut self) {
        let mut cfg_builder = &mut self.cfg_builder;
        cfg_builder.build();
    }

    fn generate_executable(&self, filename: &str) -> Result<(), Error> {
        let bitcode_fname = &format!("{}.bc", filename);
        let asm_fname = &format!("{}.s", filename);

        let mut bitcode_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(bitcode_fname)?;

        let mut asm_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(asm_fname)?;

        let bitcode_path = std::path::Path::new(bitcode_fname);
        self.module.write_bitcode_to_path(bitcode_path);

        let llc_output = Command::new("llc")
            .arg(bitcode_fname)
            .arg("-o")
            .arg(asm_fname)
            .output()
            .expect("Failed to execute command");

        if !llc_output.status.success() {
            let stderr = String::from_utf8_lossy(&llc_output.stderr);
            eprintln!("Error: {}", stderr);
        }

        let clang_output = Command::new("clang")
            .arg("-no-pie")
            .arg(asm_fname)
            .arg("-o")
            .arg(filename)
            .output()
            .expect("Failed to execute command");

        if !clang_output.status.success() {
            let stderr = String::from_utf8_lossy(&llc_output.stderr);
            eprintln!("Error: {}", stderr);
        }

        remove_file(bitcode_fname)?;
        remove_file(asm_fname)?;
        Ok(())
    }

    fn generate_llvm_bitcode(&self, filename: &str) -> Result<(), Error> {
        let bitcode_file_name = &format!("{}.bc", filename);
        let mut bitcode_file = OpenOptions::new()
            .write(true) // Enable writing to the file
            .create(true) // Create the file if it doesn't exist
            .open(bitcode_file_name)?;

        let mut bitcode_path = std::path::Path::new(bitcode_file_name);
        self.module.write_bitcode_to_path(bitcode_path);
        Ok(())
    }

    fn generate_llvm_ir(&self, filename: &str) -> Result<(), Error> {
        let llvm_ir_file_name = &format!("{}.ll", filename);
        let llvm_ir_path = std::path::Path::new(llvm_ir_file_name);
        let mut llvm_ir_file = OpenOptions::new()
            .write(true) // Enable writing to the file
            .create(true) // Create the file if it doesn't exist
            .open(llvm_ir_path)?;

        llvm_ir_file.write_all(self.module.print_to_string().to_bytes())?;
        Ok(())
    }

    fn generate(
        &mut self,
        save_file: &str,
        opt_level: OptimizationLevel,
        warn_nt: bool,
        show_cfg_size: bool,
        options: SaveOptions,
    ) -> Result<(), Error> {

        if let Some(settings) = self.settings.partial_eval_settings {
            let InterpSettings = InterpSettings::partial_evaluation(max_steps, codel_settings);
            let interpreter = Interpreter::new(
                self.cfg_builder.get_program(),
                InterpSettings::default()
            );
        }

        self.generate_cfg();
        let cfg = self.cfg_builder.get_cfg();

        if show_cfg_size {
            match env::consts::OS {
                "linux" => {
                    println!("\x1B[1;37mpietcc:\x1B[0m \x1B[1;96minfo: \x1B[0mgenerated CFG with size {}", cfg.len())
                }
                _ => {
                    println!("pietcc: info: generated CFG with size {}", cfg.len())
                }
            }
        }

        if self.cfg_builder.determine_runs_forever() {
            match env::consts::OS {
                "linux" => {
                    eprintln!("\x1B[1;37mpietcc:\x1B[0m \x1B[1;93mwarning:\x1B[0m every node in program CFG has nonzero outdegree.  This implies nontermination!")
                }
                _ => {
                    eprintln!("pietcc: warning: every node in program CFG has nonzero outdegree.  This implies nontermination!")
                }
            }
        }

        self.build_globals();
        self.build_stdout_unbuffered();
        self.build_print_stack();
        self.build_terminate();
        self.build_stack_size_check();
        self.build_binops(Instruction::Add);
        self.build_binops(Instruction::Sub);
        self.build_binops(Instruction::Div);
        self.build_binops(Instruction::Mul);
        self.build_binops(Instruction::Mod);
        self.build_binops(Instruction::Gt);
        self.build_input(Instruction::CharIn);
        self.build_input(Instruction::IntIn);
        self.build_output(Instruction::CharOut);
        self.build_output(Instruction::IntOut);
        self.build_roll();
        self.build_dup();
        self.build_push();
        self.build_pop();
        self.build_not();
        self.build_switch();
        self.build_rotate();
        self.build_retry();
        self.build_entry(&cfg);
        self.build_main();
        let config = InitializationConfig::default();
        Target::initialize_native(&config).unwrap();

        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(opt_level);

        let module_pm = PassManager::create(());
        pass_manager_builder.populate_module_pass_manager(&module_pm);

        let res = module_pm.run_on(&self.module);

        match options {
            SaveOptions::EmitExecutable => self.generate_executable(save_file),
            SaveOptions::EmitLLVMBitcode => self.generate_llvm_bitcode(save_file),
            SaveOptions::EmitLLVMIR => self.generate_llvm_ir(save_file),
        }
    }

    pub fn run(&mut self, settings: CompilerSettings) -> Result<(), Error> {
        self.generate(
            settings.output_fname,
            settings.llvm_opt_level,
            settings.warn_nt,
            settings.show_cfg_size,
            settings.save_options,
        )?;
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use inkwell::{builder::Builder, context::Context, module::Module};
    use parser::{convert::UnknownPixelSettings, loader::Loader};
    use std::fs;
    use types::program::Program;

    const SETTINGS: UnknownPixelSettings = UnknownPixelSettings::TreatAsError;
    #[test]
    fn test_entrypoint() -> std::io::Result<()> {
        let context = Context::create();
        let module = context.create_module("piet");
        let builder = context.create_builder();
        // Program
        let program = Loader::convert("../images/alpha_filled.png", SETTINGS).unwrap();
        let cfg_gen = CFGBuilder::new(&program, 1, true);
        let mut cg = CodeGen::new(&context, module, builder, cfg_gen, 1);
        let options = SaveOptions::EmitLLVMIR;
        let ir = cg.generate(
            "../../compilation.ll",
            OptimizationLevel::Aggressive,
            false,
            false,
            options,
        );
        Ok(())
    }
}
