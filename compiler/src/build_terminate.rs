use crate::codegen::CodeGen;

// Terminates the program and prints the stack
impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_terminate(&self) {
        let i64_fn_type = self.context.i64_type().fn_type(&[], false);
        let print_stack_fn = self.module.get_function("print_stack").unwrap();
        let exit_fn = self.module.get_function("exit").unwrap();

        // Constants
        let const_0 = self.context.i64_type().const_zero();
        // Function type
        let terminate_fn = self.module.add_function("terminate", i64_fn_type, None);

        // Basic blocks
        let basic_block = self.context.append_basic_block(terminate_fn, "");
        let ret_block = self.context.append_basic_block(terminate_fn, "");

        self.builder.position_at_end(basic_block);

        self.builder
            .build_call(print_stack_fn, &[], "call_print_stack");
        self.builder
            .build_call(exit_fn, &[const_0.into()], "call_exit");

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}
