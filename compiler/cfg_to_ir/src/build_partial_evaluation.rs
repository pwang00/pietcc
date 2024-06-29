use inkwell::values::AnyValue;
use interpreter::{interpreter::Interpreter, settings::InterpSettings};
use types::instruction::Instruction;
use crate::{codegen::CodeGen, consts::PIET_STACK};

impl<'a, 'b> CodeGen<'a, 'b> {

    /// Currently only supports partial evaluation for programs without input instructions (i.e. CharIn, IntIn)
    /// Analysis will eventually be extended to support those under certain circumstances
    pub(crate) fn partial_eval_to_ir(&self, settings: InterpSettings) -> Result<(), ()>{
        let mut interpreter = Interpreter::new(self.cfg_builder.get_program(), settings);
        interpreter.run();

        let state = interpreter.get_state();

        // Happens in 2 cases:
        // 1. If input instructions are encountered, which is currently unsupported
        // 2. If the program didn't terminate within the max steps
        if state.rctr != 8 && state.steps >= settings.max_steps.unwrap() {
            return Err(())
        }
        
        let stack = interpreter.get_stack();
        let push_fn = self.module.get_function(Instruction::Push.to_llvm_name()).unwrap();


        let i8_type = self.context.i8_type();
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type().fn_type(&[], false);
        let start_fn = self.module.add_function("start", void_type, None);
        let basic_block = self.context.append_basic_block(start_fn, "");

        self.builder.position_at_end(basic_block);
        // Globals
        let dp_addr = self
            .module
            .get_global("dp")
            .unwrap()
            .as_any_value_enum()
            .into_pointer_value();
        let cc_addr = self
            .module
            .get_global("cc")
            .unwrap()
            .as_any_value_enum()
            .into_pointer_value();

        let rctr_addr = self.module.get_global("rctr").unwrap().as_pointer_value();
        let const_0 = i64_type.const_zero();
        let ret_block = self.context.append_basic_block(start_fn, "ret");

        // Init (jumps to entry block)
        self.builder.position_at_end(basic_block);
        self.builder.build_unconditional_branch(entry);

        for val in stack {
            let int_val = self.context.i64_type().const_int(val as u64, false);
            self.builder.build_call(push_fn, &[int_val.into()], "");
        }

        let stdout_instrs = interpreter.get_stdout_instrs();
        let printf_fn = self.module.get_function("printf").unwrap();

        let dec_fmt = self
                .module
                .get_global("dec_fmt")
                .unwrap()
                .as_pointer_value()
                .into();

        let char_fmt = self
                .module
                .get_global("char_fmt")
                .unwrap()
                .as_pointer_value()
                .into();
        
        for (stdout_instr, val) in stdout_instrs {
            match stdout_instr {
                Instruction::CharOut => {
                    let char_val = self.context.i8_type().const_int(val as u64, false);
                    self.builder.build_call(printf_fn, &[char_fmt, char_val.into()], "");
                },
                Instruction::IntOut => {
                    let int_val = self.context.i8_type().const_int(val as u64, false);
                    self.builder.build_call(printf_fn, &[dec_fmt, int_val.into()], "");
                },
                _ => panic!("Invalid stdout instruction (must be CharOut / IntOut)")
            }
        }

        Ok(())
    }
}