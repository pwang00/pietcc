use crate::lowering_ctx::LoweringCtx;
use inkwell::IntPredicate;

pub(crate) fn build_print_stack<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
    let print_stack_fn = ctx.module.get_function("print_piet_stack").unwrap();
    let printf_fn = ctx.module.get_function("printf").unwrap();
    // Labels
    let basic_block = ctx.llvm_context.append_basic_block(print_stack_fn, "");
    let size_zero_block = ctx.llvm_context.append_basic_block(print_stack_fn, "");
    let size_gt_zero_block = ctx.llvm_context.append_basic_block(print_stack_fn, "");
    let loop_block = ctx.llvm_context.append_basic_block(print_stack_fn, "loop");
    let ret_block = ctx.llvm_context.append_basic_block(print_stack_fn, "ret");
    ctx.builder.position_at_end(basic_block);
    // Need to deref twice since LLVM globals are themselves pointers and we're operating on an i64**
    let stack_addr = ctx
        .module
        .get_global("piet_stack")
        .unwrap()
        .as_pointer_value();

    let load_piet_stack = ctx
        .builder
        .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
        .unwrap()
        .into_pointer_value();

    let index = ctx
        .builder
        .build_alloca(ctx.llvm_context.i64_type(), "index")
        .unwrap();

    ctx.builder.position_at_end(basic_block);

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let const_0 = ctx.llvm_context.i64_type().const_int(0, false);
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    let stack_size_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "stack_size")
        .unwrap()
        .into_int_value();

    let stack_fmt = ctx
        .module
        .get_global("stack_fmt")
        .unwrap()
        .as_pointer_value();

    let stack_id = ctx
        .module
        .get_global("stack_id")
        .unwrap()
        .as_pointer_value();

    let stack_id_empty = ctx
        .module
        .get_global("stack_id_empty")
        .unwrap()
        .as_pointer_value();

    let newline_fmt = ctx.module.get_global("newline").unwrap().as_pointer_value();

    let const_fmt_stack_id_gep = unsafe {
        ctx.builder
            .build_gep(stack_id.get_type(), stack_id, &[const_0, const_0], "")
            .unwrap()
    };

    let const_fmt_stack_id_empty_gep = unsafe {
        ctx.builder
            .build_gep(
                stack_id_empty.get_type(),
                stack_id_empty,
                &[const_0, const_0],
                "",
            )
            .unwrap()
    };

    let size_eq_0 = ctx
        .builder
        .build_int_compare(IntPredicate::EQ, stack_size_val, const_0, "")
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(size_eq_0, size_zero_block, size_gt_zero_block);

    ctx.builder.position_at_end(size_zero_block);

    let _ = ctx.builder.build_call(
        printf_fn,
        &[const_fmt_stack_id_empty_gep.into()],
        "call_printf_stack_const_empty",
    );
    let _ = ctx.builder.build_unconditional_branch(ret_block);

    ctx.builder.position_at_end(size_gt_zero_block);

    let _ = ctx.builder.build_call(
        printf_fn,
        &[const_fmt_stack_id_gep.into(), stack_size_val.into()],
        "call_printf_stack_const",
    );

    let const_fmt_gep = unsafe {
        ctx.builder
            .build_gep(stack_fmt.get_type(), stack_fmt, &[const_0, const_0], "")
            .unwrap()
    };
    // Store index
    let _ = ctx.builder.build_store(index, stack_size_val);
    let _ = ctx.builder.build_unconditional_branch(loop_block);
    ctx.builder.position_at_end(loop_block);

    let curr_index = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), index, "load_idx")
        .unwrap()
        .into_int_value();
    let updated_idx = ctx
        .builder
        .build_int_sub(curr_index, const_1, "decrement_stack_size")
        .unwrap();

    let top_elem = unsafe {
        ctx.builder
            .build_gep(
                load_piet_stack.get_type(),
                load_piet_stack,
                &[updated_idx],
                "fetch_curr_elem_ptr",
            )
            .unwrap()
    };

    let top_elem_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_elem, "load_elem")
        .unwrap();

    let _ = ctx.builder.build_call(
        printf_fn,
        &[const_fmt_gep.into(), top_elem_val.into()],
        "call_printf",
    );

    let _ = ctx.builder.build_store(index, updated_idx);

    let cmp = ctx
        .builder
        .build_int_compare(IntPredicate::SGT, updated_idx, const_0, "cmp_gz")
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(cmp, loop_block, ret_block);

    ctx.builder.position_at_end(ret_block);
    let newline_fmt = unsafe {
        ctx.builder
            .build_gep(newline_fmt.get_type(), newline_fmt, &[const_0, const_0], "")
            .unwrap()
    };
    let _ = ctx.builder.build_call(printf_fn, &[newline_fmt.into()], "");
    let _ = ctx.builder.build_return(None);
}
