use inkwell::IntPredicate;
use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub fn build_dup(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let not_fn_type = void_type.fn_type(&[self.context.i64_type().into()], false);
        let not_fn = self.module.add_function("piet_dup", not_fn_type, None);

        // Labels
        let basic_block = self.context.append_basic_block(not_fn, "");
        let then_block = self.context.append_basic_block(not_fn, "stack_nonempty");
        let ret_block = self.context.append_basic_block(not_fn, "ret");

        self.builder.position_at_end(basic_block);

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
        let top_ptr = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };
        let top_ptr = self.builder.build_load(top_ptr, "top_elem_ptr");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr.into_pointer_value(), "top_elem_val")
            .into_int_value();

        // Dup always increases stack size by 1
        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        let to_store = unsafe { self.builder.build_gep(stack_addr, &[stack_size_val], "") };

        // Push (dup) top element to stack
        self.builder.build_store(to_store, top_ptr_val);

        // Store updated stack size
        self.builder
            .build_store(stack_size_addr, updated_stack_size);

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}