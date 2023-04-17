use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_main(&self) {
        let main_fn_type = self.context.i64_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);

        // Call init_globals
        let init_globals = self.module.get_function("init_globals").unwrap();
        let init_block = self.context.append_basic_block(main_fn, "");
        self.builder.position_at_end(init_block);

        self.builder.build_call(init_globals, &[], "setup_stack");

        // Just testing stuff for now
        for _ in 0..5 {
            let piet_charin = self.module.get_function("piet_intin").unwrap();
            self.builder.build_call(piet_charin, &[], "call_piet_intin");
        }

        let piet_dup = self.module.get_function("piet_mod").unwrap();
        self.builder.build_call(piet_dup, &[], "call_piet_mod");

        let print_stack = self.module.get_function("print_piet_stack").unwrap();
        self.builder
            .build_call(print_stack, &[], "call_print_stack");

        self.builder
            .build_return(Some(&self.context.i64_type().const_zero()));
    }
}
