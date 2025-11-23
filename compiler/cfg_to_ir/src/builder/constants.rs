use crate::lowering_ctx::LoweringCtx;

pub(crate) fn build_constants(ctx: &LoweringCtx) {
    unsafe {
        let _ = ctx.builder.build_global_string("%ld\0", "dec_fmt");
        let _ = ctx.builder.build_global_string("%c\0", "char_fmt");
        let _ = ctx.builder.build_global_string("%s\0", "string_fmt");
        let _ = ctx.builder.build_global_string("%ld \0", "stack_fmt");
        let _ = ctx
            .builder
            .build_global_string("\nStack (size %d): ", "stack_id");
        let _ = ctx
            .builder
            .build_global_string("\nStack empty", "stack_id_empty");
    }
}
