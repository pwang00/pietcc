use crate::codegen::CodeGen;

#[allow(unused)]
impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_push(&self) {
        let void_type = self.context.void_type();
        let push_fn_type = void_type.fn_type(&[self.context.i64_type().into()], false);
        let push_fn = self.module.add_function("piet_push", push_fn_type, None);
        let basic_block = self.context.append_basic_block(push_fn, "");
        self.builder.position_at_end(basic_block);

        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let const_1 = self.context.i64_type().const_int(1, false);

        let top_ptr = self.builder.build_load(
            unsafe { self.builder.build_gep(stack_addr, &[stack_size_val], "") },
            "top_elem_ptr",
        );

        let first_param = push_fn.get_first_param().unwrap();
        self.builder
            .build_store(top_ptr.into_pointer_value(), first_param);

        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        self.builder
            .build_store(stack_size_addr, updated_stack_size);

        self.builder.build_return(None);
    }
}