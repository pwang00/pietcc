use crate::{codegen::CodeGen, consts::STACK_SIZE};

// Terminates the program and prints the stack
impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_stack_size_check(&self) {
        let void_fn_type = self.context.void_type().fn_type(&[], false);
        let stack_check_fn = self
            .module
            .add_function("stack_size_check", void_fn_type, None);
        let terminate_fn = self.module.get_function("terminate").unwrap();

        // Basic blocks
        let basic_block = self.context.append_basic_block(stack_check_fn, "");
        let stack_exhausted_block = self.context.append_basic_block(stack_check_fn, "");
        let ret_block = self.context.append_basic_block(stack_check_fn, "");

        self.builder.position_at_end(basic_block);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let max_stack_size_value = self.context.i64_type().const_int(STACK_SIZE as u64, false);

        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::UGE,
            stack_size_val,
            max_stack_size_value,
            "check_overflow",
        );

        self.builder
            .build_conditional_branch(cmp, stack_exhausted_block, ret_block);

        self.builder.position_at_end(stack_exhausted_block);
        self.builder.build_call(terminate_fn, &[], "call_terminate");
        self.builder.build_unreachable();

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub(crate) fn build_terminate(&self) {
        let i64_fn_type = self.context.i64_type().fn_type(&[], false);
        let print_stack_fn = self.module.get_function("print_piet_stack").unwrap();
        let exit_fn = self.module.get_function("exit").unwrap();
        let printf_fn = self.module.get_function("printf").unwrap();
        // Constants
        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);

        // Function type
        let terminate_fn = self.module.add_function("terminate", i64_fn_type, None);

        // Basic blocks
        let basic_block = self.context.append_basic_block(terminate_fn, "");
        self.builder.position_at_end(basic_block);
        let exhausted_fmt = self.module.get_global("exhausted_fmt").unwrap();
        let _exhausted_fmt_load = self
            .builder
            .build_load(exhausted_fmt.as_pointer_value(), "load_exhausted_fmt");
        let exhausted_fmt_gep = unsafe {
            self.builder.build_gep(
                exhausted_fmt.as_pointer_value(),
                &[const_0, const_0],
                "load_gep",
            )
        };

        self.builder
            .build_call(printf_fn, &[exhausted_fmt_gep.into()], "");
        self.builder.build_call(print_stack_fn, &[], "");
        self.builder
            .build_call(exit_fn, &[const_1.into()], "call_exit");
        self.builder.build_return(Some(&const_1));
    }
}
