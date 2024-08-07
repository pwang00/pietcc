use crate::codegen::CodeGen;
use inkwell::IntPredicate;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_print_stack(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let print_stack_fn_type = void_type.fn_type(&[], false);
        let print_stack_fn =
            self.module
                .add_function("print_piet_stack", print_stack_fn_type, None);
        let printf_fn = self.module.get_function("printf").unwrap();
        // Labels
        let basic_block = self.context.append_basic_block(print_stack_fn, "");
        let size_zero_block = self.context.append_basic_block(print_stack_fn, "");
        let size_gt_zero_block = self.context.append_basic_block(print_stack_fn, "");
        let loop_block = self.context.append_basic_block(print_stack_fn, "loop");
        let ret_block = self.context.append_basic_block(print_stack_fn, "ret");
        self.builder.position_at_end(basic_block);
        // Need to deref twice since LLVM globals are themselves pointers and we're operating on an i64**
        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let load_piet_stack = self
            .builder
            .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
            .unwrap()
            .into_pointer_value();

        let index = self.builder.build_alloca(self.context.i64_type(), "index").unwrap();

        self.builder.position_at_end(basic_block);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_size_val = self
            .builder
            .build_load(self.context.i64_type(), stack_size_addr, "stack_size")
            .unwrap()
            .into_int_value();

        let stack_fmt = self
            .module
            .get_global("stack_fmt")
            .unwrap()
            .as_pointer_value();

        let stack_id = self
            .module
            .get_global("stack_id")
            .unwrap()
            .as_pointer_value();

        let stack_id_empty = self
            .module
            .get_global("stack_id_empty")
            .unwrap()
            .as_pointer_value();

        let newline_fmt = self
            .module
            .get_global("newline")
            .unwrap()
            .as_pointer_value();

        let const_fmt_stack_id_gep =
            unsafe { self.builder.build_gep(stack_id.get_type(), stack_id, &[const_0, const_0], "").unwrap() };

        let const_fmt_stack_id_empty_gep = unsafe {
            self.builder
                .build_gep(stack_id_empty.get_type(), stack_id_empty, &[const_0, const_0], "").unwrap()
        };

        let size_eq_0 =
            self.builder
                .build_int_compare(IntPredicate::EQ, stack_size_val, const_0, "")
                .unwrap();
        let _ = self.builder
            .build_conditional_branch(size_eq_0, size_zero_block, size_gt_zero_block);

        self.builder.position_at_end(size_zero_block);

        let _ = self.builder.build_call(
            printf_fn,
            &[const_fmt_stack_id_empty_gep.into()],
            "call_printf_stack_const_empty",
        );
        let _ = self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(size_gt_zero_block);

        let _ = self.builder.build_call(
            printf_fn,
            &[const_fmt_stack_id_gep.into(), stack_size_val.into()],
            "call_printf_stack_const",
        );

        let const_fmt_gep = unsafe { self.builder.build_gep(stack_fmt.get_type(), stack_fmt, &[const_0, const_0], "").unwrap() };
        // Store index
        let _ = self.builder.build_store(index, stack_size_val);
        let _ = self.builder.build_unconditional_branch(loop_block);
        self.builder.position_at_end(loop_block);

        let curr_index = self.builder.build_load(self.context.i64_type(), index, "load_idx").unwrap()
            .into_int_value();
        let updated_idx = self
            .builder
            .build_int_sub(curr_index, const_1, "decrement_stack_size").unwrap();

        let top_elem = unsafe {
            self.builder
                .build_gep(load_piet_stack.get_type(), load_piet_stack, &[updated_idx], "fetch_curr_elem_ptr")
                .unwrap()
        };

        let top_elem_val = self.builder.build_load(self.context.i64_type(), top_elem, "load_elem").unwrap();

        let _ = self.builder.build_call(
            printf_fn,
            &[const_fmt_gep.into(), top_elem_val.into()],
            "call_printf",
        );

        let _ = self.builder.build_store(index, updated_idx);

        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SGT, updated_idx, const_0, "cmp_gz").unwrap();
        let _ = self.builder
            .build_conditional_branch(cmp, loop_block, ret_block);

        self.builder.position_at_end(ret_block);
        let newline_fmt = unsafe { self.builder.build_gep(newline_fmt.get_type(), newline_fmt, &[const_0, const_0], "").unwrap() };
        let _ = self.builder
            .build_call(printf_fn, &[newline_fmt.into()], "");
        let _ = self.builder.build_return(None);
    }
}
