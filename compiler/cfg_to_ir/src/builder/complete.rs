use crate::lowering_ctx::LoweringCtx;
use piet_core::state::ExecutionState;

pub(crate) fn build_complete<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, execution_state: &ExecutionState) {
    // Get start function
    // Insert basic block after
    let start_fn = ctx.module.get_function("start").unwrap();
    let printf_fn = ctx.module.get_function("printf").unwrap();
    let print_stdout_basic_block = ctx
        .llvm_context
        .append_basic_block(start_fn, "print_stdout");
    ctx.builder.position_at_end(print_stdout_basic_block);
    let stdout: String = execution_state
        .stdout
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("");

    let result_global_str = unsafe {
        ctx.builder
            .build_global_string(&stdout, "result")
            .unwrap()
            .as_pointer_value()
    };
    let string_fmt = ctx
        .module
        .get_global("string_fmt")
        .unwrap()
        .as_pointer_value();
    let _ = ctx.builder.build_call(
        printf_fn,
        &[string_fmt.into(), result_global_str.into()],
        "call_printf_result",
    );
    let ret = ctx.builder.build_return(None);
}
