use crate::{consts::STACK_SIZE, lowering_ctx::LoweringCtx};

// Terminates the program and prints the stack
pub(crate) fn build_stack_size_check<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let void_fn_type = ctx.llvm_context.void_type().fn_type(&[], false);
    let stack_check_fn = ctx
        .module
        .add_function("stack_size_check", void_fn_type, None);
    let terminate_fn = ctx.module.get_function("terminate").unwrap();

    // Basic blocks
    let basic_block = ctx.llvm_context.append_basic_block(stack_check_fn, "");
    let stack_exhausted_block = ctx.llvm_context.append_basic_block(stack_check_fn, "");
    let ret_block = ctx.llvm_context.append_basic_block(stack_check_fn, "");

    ctx.builder.position_at_end(basic_block);

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let stack_size_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "stack_size")
        .unwrap()
        .into_int_value();

    let max_stack_size_value = ctx.llvm_context.i64_type().const_int(STACK_SIZE as u64, false);

    let cmp = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::UGE,
            stack_size_val,
            max_stack_size_value,
            "check_overflow",
        )
        .unwrap();

    let _ = ctx
        .builder
        .build_conditional_branch(cmp, stack_exhausted_block, ret_block);

    ctx.builder.position_at_end(stack_exhausted_block);
    let _ = ctx.builder.build_call(terminate_fn, &[], "call_terminate");
    let _ = ctx.builder.build_unreachable();

    ctx.builder.position_at_end(ret_block);
    let _ = ctx.builder.build_return(None);
}

pub(crate) fn build_terminate<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let i64_fn_type = ctx.llvm_context.i64_type().fn_type(&[], false);
    let print_stack_fn = ctx.module.get_function("print_piet_stack").unwrap();
    let exit_fn = ctx.module.get_function("exit").unwrap();
    let printf_fn = ctx.module.get_function("printf").unwrap();
    // Constants
    let const_0 = ctx.llvm_context.i64_type().const_zero();
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    // Function type
    let terminate_fn = ctx.module.add_function("terminate", i64_fn_type, None);

    // Basic blocks
    let basic_block = ctx.llvm_context.append_basic_block(terminate_fn, "");
    ctx.builder.position_at_end(basic_block);
    let exhausted_fmt = ctx.module.get_global("exhausted_fmt").unwrap();
    let _exhausted_fmt_load = ctx.builder.build_load(
        exhausted_fmt.as_pointer_value().get_type(),
        exhausted_fmt.as_pointer_value(),
        "load_exhausted_fmt",
    );
    let exhausted_fmt_gep = unsafe {
        ctx.builder
            .build_gep(
                exhausted_fmt.as_pointer_value().get_type(),
                exhausted_fmt.as_pointer_value(),
                &[const_0, const_0],
                "load_gep",
            )
            .unwrap()
    };

    let _ = ctx
        .builder
        .build_call(printf_fn, &[exhausted_fmt_gep.into()], "");
    let _ = ctx.builder.build_call(print_stack_fn, &[], "");
    let _ = ctx
        .builder
        .build_call(exit_fn, &[const_1.into()], "call_exit");
    let _ = ctx.builder.build_return(Some(&const_1));
}
