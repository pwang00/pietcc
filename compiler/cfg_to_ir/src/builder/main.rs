use crate::lowering_ctx::LoweringCtx;

pub(crate) fn build_main<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let main_fn = ctx.module.get_function("main").unwrap();
    let start_fn = ctx.module.get_function("start").unwrap();
    let print_stack_fn = ctx.module.get_function("print_piet_stack").unwrap();
    let set_stdout_unbuffered_fn = ctx.module.get_function("set_stdout_unbuffered").unwrap();
    let init_block = ctx.llvm_context.append_basic_block(main_fn, "");

    if let Some(init_globals_fn) = ctx.module.get_function("init_globals") {
        ctx.builder.position_at_end(init_block);
        let _ = ctx
            .builder
            .build_call(init_globals_fn, &[], "setup_globals");
    }

    let _ = ctx
        .builder
        .build_call(set_stdout_unbuffered_fn, &[], "set_stdout_unbuffered");
    let _ = ctx.builder.build_call(start_fn, &[], "start");

    let _ = ctx
        .builder
        .build_call(print_stack_fn, &[], "print_stack_fn");
    let _ = ctx
        .builder
        .build_return(Some(&ctx.llvm_context.i64_type().const_zero()));
}
