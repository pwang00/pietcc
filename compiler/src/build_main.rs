use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_main(&self) {
        let main_fn_type = self.context.i64_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);

        // Call init_globals
        let init_globals = self.module.get_function("init_globals").unwrap();
        let start = self.module.get_function("start").unwrap();
        let print_stack = self.module.get_function("print_piet_stack").unwrap();
        let init_block = self.context.append_basic_block(main_fn, "");

        self.builder.position_at_end(init_block);

        self.builder.build_call(init_globals, &[], "setup_globals");

        self.builder.build_call(start, &[], "start");

        self.builder.build_call(print_stack, &[], "print_stack");
        self.builder
            .build_return(Some(&self.context.i64_type().const_zero()));
    }
}
