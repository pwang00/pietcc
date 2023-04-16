use inkwell::IntPredicate;
use types::instruction::Instruction;

use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_input(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let in_fn_type = void_type.fn_type(&[], false);

        let in_fn = match instr {
            Instruction::IntIn => self.module.add_function("piet_intin", in_fn_type, None),
            Instruction::CharIn => self.module.add_function("piet_charin", in_fn_type, None),
            _ => panic!("Not an input instruction!"),
        };

        // Labels
        let basic_block = self.context.append_basic_block(in_fn, "");
        let ret_block = self.context.append_basic_block(in_fn, "ret");

        self.builder.position_at_end(basic_block);
        let read_addr = self
            .builder
            .build_alloca(self.context.i64_type(), "stack_alloc");

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

        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);

        let const_fmt_gep = unsafe { self.builder.build_gep(fmt, &[const_0, const_0], "") };

        let scanf_fn = self.module.get_function("scanf").unwrap();
        let scanf =
            self.builder
                .build_call(scanf_fn, &[const_fmt_gep.into(), read_addr.into()], "scanf");

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let push_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[stack_size_val], "") };
        let push_ptr = self.builder.build_load(push_ptr_gep, "push_elem_ptr");

        let result = self.builder.build_load(read_addr, "scanf_elem");
        self.builder
            .build_store(push_ptr.into_pointer_value(), result);

        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        // Store updated stack size
        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8).ok();

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub fn build_output(&self, instr: Instruction) {
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

        let const_0 = self.context.i64_type().const_zero();

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

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
        let top_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };
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

        let top_ptr_deref1 = self.builder.build_load(top_ptr_gep, "top_elem_deref1");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr_deref1.into_pointer_value(), "top_elem_val");

        let const_fmt_gep = unsafe { self.builder.build_gep(fmt, &[const_0, const_0], "") };

        let printf = self.builder.build_call(
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