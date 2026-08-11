use crate::lowering_ctx::LoweringCtx;
use piet_core::state::ExecutionState;

pub(crate) fn build_complete(ctx: &LoweringCtx, execution_state: &ExecutionState) {
    // Get start function
    // Insert basic block after
    let main_fn = ctx.module.get_function("main").unwrap();
    let printf_fn = ctx.module.get_function("printf").unwrap();
    let print_stdout_basic_block = ctx.llvm_context.append_basic_block(main_fn, "print_stdout");
    ctx.builder.position_at_end(print_stdout_basic_block);
    let stdout: String = execution_state
        .stdout
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("");

    let stack_vals: String = execution_state
        .stack
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");

    let result_global_str = unsafe {
        ctx.builder
            .build_global_string(&stdout, "result")
            .unwrap()
            .as_pointer_value()
    };

    let stack_vals_global_str = unsafe {
        ctx.builder
            .build_global_string(&stack_vals, "stack_vals")
            .unwrap()
            .as_pointer_value()
    };

    let string_fmt = ctx
        .module
        .get_global("string_fmt")
        .unwrap()
        .as_pointer_value();

    unsafe {
        ctx.builder.build_global_string(
            &format!("\nStack (size {}): %s\n", stack_vals.len()),
            "stack_nonempty",
        )
    }
    .unwrap();

    let stack_id_fmt = if stack_vals.is_empty() {
        ctx.module
            .get_global("stack_id_empty")
            .unwrap()
            .as_pointer_value()
    } else {
        ctx.module
            .get_global("stack_nonempty")
            .unwrap()
            .as_pointer_value()
    };

    ctx.builder
        .build_call(
            printf_fn,
            &[string_fmt.into(), result_global_str.into()],
            "call_printf_result",
        )
        .unwrap();

    ctx.builder
        .build_call(
            printf_fn,
            &[stack_id_fmt.into(), stack_vals_global_str.into()],
            "print_stack",
        )
        .unwrap();

    ctx.builder
        .build_return(Some(&ctx.llvm_context.i64_type().const_int(0, false)))
        .unwrap();
}
