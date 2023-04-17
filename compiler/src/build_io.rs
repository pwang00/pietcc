use inkwell::{
    values::{BasicValue, IntValue},
    IntPredicate,
};
use types::instruction::Instruction;

use crate::codegen::CodeGen;

#[allow(unused)]
impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_input(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let in_fn_type = void_type.fn_type(&[], false);

        let in_fn = match instr {
            Instruction::IntIn => self.module.add_function("piet_intin", in_fn_type, None),
            Instruction::CharIn => self.module.add_function("piet_charin", in_fn_type, None),
            _ => panic!("Not an input instruction!"),
        };

        // Consts
        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);

        // Labels
        let basic_block = self.context.append_basic_block(in_fn, "");
        self.builder.position_at_end(basic_block);

        // Local variable to store our input
        let read_addr = self
            .builder
            .build_alloca(self.context.i64_type(), "stack_alloc");

        // The stack may not necessarily be zero'd out, which may cause problems when printing
        // Since %c only reads in at one-byte boundaries, if the higher bits of our value are nonzero
        // Then printf("%c", val) could print garbage and we would like this not to happen
        self.builder.build_store(read_addr, const_0);

        let fmt = match instr {
            Instruction::IntIn => self
                .module
                .get_global("dec_fmt")
                .unwrap()
                .as_pointer_value(),
            Instruction::CharIn => self
                .module
                .get_global("char_fmt")
                .unwrap()
                .as_pointer_value(),
            _ => panic!("Not an output instruction"),
        };

        // %ld or %c
        let const_fmt_gep = unsafe { self.builder.build_gep(fmt, &[const_0, const_0], "") };

        // Build scanf call
        // scanf reads into our local variable, so we need to load it next
        let scanf_fn = self.module.get_function("__isoc99_scanf").unwrap();
        let scanf =
            self.builder
                .build_call(scanf_fn, &[const_fmt_gep.into(), read_addr.into()], "scanf");

        // Loads local var and sets alignment
        let load_scanf_elem = self
            .builder
            .build_load(read_addr, "scanf_elem")
            .as_instruction_value()
            .unwrap();

        load_scanf_elem.set_alignment(8);

        let x: IntValue = load_scanf_elem.try_into().unwrap();
        let result = self
            .builder
            .build_int_s_extend(x, self.context.i64_type(), "sext_to_i64");

        // &stack_size
        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let stack_size_load_instr = self
            .builder
            .build_load(stack_size_addr, "load_stack_size")
            .as_instruction_value()
            .unwrap();

        // For some reason Inkwell aligns i64s at a 4 byte boundary and not 8 byte, very weirdga
        stack_size_load_instr.set_alignment(8);
        let stack_size_val: IntValue = stack_size_load_instr.try_into().unwrap();

        // &piet_stack
        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let load_piet_stack = self
            .builder
            .build_load(stack_addr, "load_piet_stack")
            .into_pointer_value();

        // Push to stack
        let push_ptr_gep = unsafe {
            self.builder
                .build_gep(load_piet_stack, &[stack_size_val], "top_elem_addr")
        };

        let store_to_stack = self.builder.build_store(push_ptr_gep, result);

        store_to_stack.set_alignment(8);

        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        // Store updated stack size
        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8).ok();
        self.builder.build_return(None);
    }

    pub(crate) fn build_output(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let out_fn_type = void_type.fn_type(&[], false);

        let out_fn = match instr {
            Instruction::IntOut => self.module.add_function("piet_intout", out_fn_type, None),
            Instruction::CharOut => self.module.add_function("piet_charout", out_fn_type, None),
            _ => panic!("Not an output instruction!"),
        };

        // Labels
        let basic_block = self.context.append_basic_block(out_fn, "");
        let then_block = self.context.append_basic_block(out_fn, "stack_nonempty");
        let ret_block = self.context.append_basic_block(out_fn, "ret");

        self.builder.position_at_end(basic_block);

        // Constants
        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let stack_size_load_instr = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .as_instruction_value()
            .unwrap();

        // For some reason Inkwell aligns i64s at a 4 byte boundary and not 8 byte, very weirdga
        stack_size_load_instr.set_alignment(8);

        let stack_size_val = stack_size_load_instr.try_into().unwrap();

        let stack_size_cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        );

        self.builder
            .build_conditional_branch(stack_size_cmp, then_block, ret_block);
        self.builder.position_at_end(then_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx");

        // Need to deref twice since LLVM globals are themselves pointers and we're operating on an i64**
        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let load_piet_stack = self
            .builder
            .build_load(stack_addr, "load_piet_stack")
            .into_pointer_value();

        let top_ptr_gep = unsafe { self.builder.build_gep(load_piet_stack, &[top_idx], "") };
        let printf_fn = self.module.get_function("printf").unwrap();

        let fmt = match instr {
            Instruction::IntOut => self
                .module
                .get_global("dec_fmt")
                .unwrap()
                .as_pointer_value()
                .into(),
            Instruction::CharOut => self
                .module
                .get_global("char_fmt")
                .unwrap()
                .as_pointer_value()
                .into(),
            _ => panic!("Not an output instruction"),
        };

        let top_ptr_load_instr = self
            .builder
            .build_load(top_ptr_gep, "top_elem_val")
            .as_instruction_value()
            .unwrap();

        top_ptr_load_instr.set_alignment(8);

        let top_ptr_val: IntValue = top_ptr_load_instr.try_into().unwrap();

        let const_fmt_gep = unsafe { self.builder.build_gep(fmt, &[const_0, const_0], "") };

        let _printf = self.builder.build_call(
            printf_fn,
            &[const_fmt_gep.into(), top_ptr_val.into()],
            "printf",
        );

        let updated_stack_size =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size");

        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8).ok();
        self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}
