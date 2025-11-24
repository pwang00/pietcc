use crate::lowering_ctx::LoweringCtx;
use inkwell::AddressSpace;
// ...

pub(crate) fn build_stdout_unbuffered<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let set_stdout_unbuffered_fn = ctx.module.get_function("set_stdout_unbuffered").unwrap();

    // Basic blocks
    let basic_block = ctx.llvm_context.append_basic_block(set_stdout_unbuffered_fn, "");
    // Constants
    let const_0 = ctx.llvm_context.i32_type().const_zero();
    let setvbuf_fn = ctx.module.get_function("setvbuf").unwrap();
    let fdopen_fn = ctx.module.get_function("fdopen").unwrap();
    let stdout_fd = ctx.llvm_context.i32_type().const_int(1, false); // File descriptor for stdout is 1
    let fdopen_mode = ctx
        .module
        .get_global("fdopen_mode")
        .unwrap()
        .as_pointer_value();
    let fdopen_fmt_gep = unsafe {
        ctx.builder
            .build_gep(fdopen_mode.get_type(), fdopen_mode, &[const_0, const_0], "")
            .unwrap()
    };
    ctx.builder.position_at_end(basic_block);

    let stdout_file_ptr = ctx
        .builder
        .build_call(
            fdopen_fn,
            &[stdout_fd.into(), fdopen_fmt_gep.into()],
            "stdout",
        )
        .unwrap()
        .try_as_basic_value()
        .unwrap_left()
        .into_pointer_value();

    let null_ptr = ctx.llvm_context.ptr_type(AddressSpace::default()).const_null();
    let unbuffered = ctx.llvm_context.i32_type().const_int(2, false);

    ctx.builder
        .build_call(
            setvbuf_fn,
            &[
                stdout_file_ptr.into(),
                null_ptr.into(),
                unbuffered.into(),
                const_0.into(),
            ],
            "",
        )
        .ok();

    ctx.builder.build_return(None).ok();
}
