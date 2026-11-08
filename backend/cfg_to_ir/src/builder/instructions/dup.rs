use crate::lowering_ctx::LoweringCtx;
use inkwell::{
    values::{BasicValue, PointerValue},
    IntPredicate,
};
use piet_core::instruction::Instruction;

#[allow(unused)]
pub(crate) fn build_dup<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
    let dup_fn = ctx.module.get_function(Instruction::Dup.to_llvm_name()).unwrap();

    // Labels
    let basic_block = ctx.llvm_context.append_basic_block(dup_fn, "");
    let then_block = ctx
        .llvm_context
        .append_basic_block(dup_fn, "stack_nonempty");
    let ret_block = ctx.llvm_context.append_basic_block(dup_fn, "ret");

    ctx.builder.position_at_end(basic_block);

    let stack_size_check_fn = ctx.module.get_function("stack_size_check").unwrap();
    ctx.builder
        .build_call(stack_size_check_fn, &[], "call_stack_size_check");

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

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

    // Need to deref twice since LLVM globals are themselves pointers and we're operating on an i64**
    let load_piet_stack = ctx
        .builder
        .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
        .unwrap()
        .as_instruction_value()
        .unwrap();

    load_piet_stack.set_alignment(8);

    let load_piet_stack: PointerValue = load_piet_stack.try_into().unwrap();

    let top_ptr = unsafe {
        ctx.builder
            .build_gep(load_piet_stack.get_type(), load_piet_stack, &[top_idx], "")
            .unwrap()
    };
    let top_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr, "top_elem_ptr")
        .unwrap();

    let new_top_ptr = unsafe {
        ctx.builder
            .build_gep(
                ctx.llvm_context.i64_type(),
                load_piet_stack,
                &[stack_size_val],
                "",
            )
            .unwrap()
    };

    // Dup always increases stack size by 1
    let updated_stack_size = ctx
        .builder
        .build_int_add(stack_size_val, const_1, "increment_stack_size")
        .unwrap();

    // Push (dup) top element to stack
    ctx.builder.build_store(new_top_ptr, top_ptr_val);

    // Store updated stack size
    ctx.builder.build_store(stack_size_addr, updated_stack_size);

    ctx.builder.build_unconditional_branch(ret_block);
    ctx.builder.position_at_end(ret_block);
    ctx.builder.build_return(None);
}
