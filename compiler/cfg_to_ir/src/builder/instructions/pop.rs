use inkwell::IntPredicate;
use piet_core::instruction::Instruction;

use crate::lowering_ctx::LoweringCtx;

#[allow(unused)]
pub(crate) fn build_pop<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
    let void_type = ctx.llvm_context.void_type();
    let pop_fn_type = void_type.fn_type(&[], false);
    let pop_fn = ctx
        .module
        .add_function(Instruction::Pop.to_llvm_name(), pop_fn_type, None);

    // Labels
    let basic_block = ctx.llvm_context.append_basic_block(pop_fn, "");
    let then_block = ctx.llvm_context.append_basic_block(pop_fn, "stack_noempty");
    let ret_block = ctx.llvm_context.append_basic_block(pop_fn, "ret");

    ctx.builder.position_at_end(basic_block);

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    let stack_size_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "stack_size")
        .unwrap()
        .into_int_value();

    let cmp = ctx
        .builder
        .build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        )
        .unwrap();

    ctx.builder
        .build_conditional_branch(cmp, then_block, ret_block);

    ctx.builder.position_at_end(then_block);

    let updated_stack_size = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
        .unwrap();

    let store = ctx
        .builder
        .build_store(stack_size_addr, updated_stack_size)
        .unwrap();

    store.set_alignment(8).ok();
    ctx.builder.build_unconditional_branch(ret_block);

    ctx.builder.position_at_end(ret_block);
    ctx.builder.build_return(None);
}
