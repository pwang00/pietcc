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

        let piet_charin = self.module.get_function("piet_intin").unwrap();
        self.builder
            .build_call(piet_charin, &[], "call_piet_charin");

        let piet_charin = self.module.get_function("piet_intin").unwrap();
        self.builder
            .build_call(piet_charin, &[], "call_piet_charin1");

        let piet_charout = self.module.get_function("piet_intout").unwrap();
        self.builder
            .build_call(piet_charout, &[], "call_piet_charout");

        let piet_charout = self.module.get_function("piet_intout").unwrap();
        self.builder
            .build_call(piet_charout, &[], "call_piet_charout1");

        self.builder
            .build_return(Some(&self.context.i64_type().const_zero()));
    }
}
