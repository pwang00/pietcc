use inkwell::IntPredicate;
use types::instruction::Instruction;

use crate::codegen::CodeGen;

#[allow(unused)]
impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_pop(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let pop_fn_type = void_type.fn_type(&[], false);
        let pop_fn = self
            .module
            .add_function(Instruction::Pop.to_llvm_name(), pop_fn_type, None);

        // Labels
        let basic_block = self.context.append_basic_block(pop_fn, "");
        let then_block = self.context.append_basic_block(pop_fn, "stack_noempty");
        let ret_block = self.context.append_basic_block(pop_fn, "ret");

        self.builder.position_at_end(basic_block);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_size_val = self
            .builder
            .build_load(self.context.i64_type(), stack_size_addr, "stack_size")
            .unwrap()
            .into_int_value();

        let cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        ).unwrap();

        self.builder
            .build_conditional_branch(cmp, then_block, ret_block);

        self.builder.position_at_end(then_block);

        let updated_stack_size =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
                .unwrap();

        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size)
            .unwrap();

        store.set_alignment(8).ok();
        self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}
