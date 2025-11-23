use inkwell::IntPredicate;
use piet_core::instruction::Instruction;

use crate::lowering_ctx::LoweringCtx;

#[allow(unused)]
pub(crate) fn build_not<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
    let void_type = ctx.llvm_context.void_type();
    let not_fn_type = void_type.fn_type(&[], false);
    let not_fn = ctx
        .module
        .add_function(Instruction::Not.to_llvm_name(), not_fn_type, None);

    // Labels
    let basic_block = ctx.llvm_context.append_basic_block(not_fn, "");
    let then_block = ctx.llvm_context.append_basic_block(not_fn, "stack_nonempty");
    let ret_block = ctx.llvm_context.append_basic_block(not_fn, "ret");

    ctx.builder.position_at_end(basic_block);

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let const_0 = ctx.llvm_context.i64_type().const_int(0, false);
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    let stack_addr = ctx
        .module
        .get_global("piet_stack")
        .unwrap()
        .as_pointer_value();

    let stack_size_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "stack_size")
        .unwrap()
        .into_int_value();

    let stack_size_cmp = ctx
        .builder
        .build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        )
        .unwrap();

    ctx.builder
        .build_conditional_branch(stack_size_cmp, then_block, ret_block);
    ctx.builder.position_at_end(then_block);
    let top_idx = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "top_elem_idx")
        .unwrap();

    let load_piet_stack = ctx
        .builder
        .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
        .unwrap()
        .into_pointer_value();
    let top_ptr = unsafe {
        ctx.builder
            .build_gep(
                load_piet_stack.get_type(),
                load_piet_stack,
                &[top_idx],
                "top_elem_ptr",
            )
            .unwrap()
    };

    let top_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr, "top_elem_val")
        .unwrap()
        .into_int_value();

    let value_cmp = ctx
        .builder
        .build_int_compare(IntPredicate::EQ, top_ptr_val, const_0, "top_value_is_zero")
        .unwrap();

    let zext_cmp = ctx
        .builder
        .build_int_z_extend(value_cmp, ctx.llvm_context.i64_type(), "zero_extend_cmp")
        .unwrap();

    ctx.builder.build_store(top_ptr, zext_cmp);

    ctx.builder.build_unconditional_branch(ret_block);
    ctx.builder.position_at_end(ret_block);
    ctx.builder.build_return(None);
}
