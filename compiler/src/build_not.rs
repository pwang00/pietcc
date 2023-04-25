use inkwell::IntPredicate;
use types::instruction::Instruction;

use crate::codegen::CodeGen;

#[allow(unused)]
impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_not(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let not_fn_type = void_type.fn_type(&[], false);
        let not_fn = self
            .module
            .add_function(Instruction::Not.to_llvm_name(), not_fn_type, None);

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

        let const_0 = self.context.i64_type().const_int(0, false);
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

        let load_piet_stack = self
            .builder
            .build_load(stack_addr, "load_piet_stack")
            .into_pointer_value();
        let top_ptr = unsafe {
            self.builder
                .build_gep(load_piet_stack, &[top_idx], "top_elem_ptr")
        };

        let top_ptr_val = self
            .builder
            .build_load(top_ptr, "top_elem_val")
            .into_int_value();

        let value_cmp = self.builder.build_int_compare(
            IntPredicate::EQ,
            top_ptr_val,
            const_0,
            "top_value_is_zero",
        );

        let zext_cmp =
            self.builder
                .build_int_z_extend(value_cmp, self.context.i64_type(), "zero_extend_cmp");

        self.builder.build_store(top_ptr, zext_cmp);

        self.builder.build_unconditional_branch(ret_block);
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}
