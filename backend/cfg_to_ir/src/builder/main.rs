use piet_core::state::ExecutionState;

use crate::{consts::STACK_SIZE, lowering_ctx::LoweringCtx};

pub(crate) fn build_main(ctx: &LoweringCtx, execution_state: &ExecutionState) {
    let main_fn = ctx.module.get_function("main").unwrap();
    let start_fn = ctx.module.get_function("start").unwrap();
    let print_stack_fn = ctx.module.get_function("print_piet_stack").unwrap();
    let initialize_piet_stack_fn = ctx.module.get_function("initialize_piet_stack").unwrap();
    let set_stdout_unbuffered_fn = ctx.module.get_function("set_stdout_unbuffered").unwrap();
    let init_block = ctx.llvm_context.append_basic_block(main_fn, "");

    if let Some(init_globals_fn) = ctx.module.get_function("init_globals") {
        ctx.builder.position_at_end(init_block);
        ctx.builder
            .build_call(init_globals_fn, &[], "setup_globals")
            .unwrap();
    }
    let malloc_fn = ctx.module.get_function("malloc").unwrap();

    let size_value = ctx
        .llvm_context
        .i64_type()
        .const_int((STACK_SIZE * 8) as u64, false);

    let malloc_call = ctx
        .builder
        .build_call(malloc_fn, &[size_value.into()], "malloc");

    let value = malloc_call.unwrap().try_as_basic_value().unwrap_basic();

    let piet_stack = ctx.module.get_global("piet_stack").unwrap();

    ctx.builder
        .build_store(piet_stack.as_pointer_value(), value.into_pointer_value())
        .unwrap();

    ctx.builder
        .build_call(set_stdout_unbuffered_fn, &[], "set_stdout_unbuffered")
        .unwrap();

    // Partial execution result
    if execution_state.steps > 0 && !execution_state.stack.is_empty() {
        ctx.builder
            .build_call(initialize_piet_stack_fn, &[], "initialize_piet_stack")
            .unwrap();
    }

    ctx.builder.build_call(start_fn, &[], "start").unwrap();

    ctx.builder
        .build_call(print_stack_fn, &[], "print_stack_fn")
        .unwrap();

    ctx.builder
        .build_return(Some(&ctx.llvm_context.i64_type().const_zero()))
        .unwrap();
}
