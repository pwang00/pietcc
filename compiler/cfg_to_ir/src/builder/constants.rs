use crate::lowering_ctx::LoweringCtx;
use piet_core::instruction::Instruction;
use strum::IntoEnumIterator;

pub(crate) fn build_constants(ctx: &LoweringCtx) {
    unsafe {
        // Format strings for I/O
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

        // Input prompt strings
        let _ = ctx
            .builder
            .build_global_string("Enter number: ", "input_message_int");
        let _ = ctx
            .builder
            .build_global_string("Enter char: ", "input_message_char");

        let _ = ctx.builder.build_global_string("w", "fdopen_mode");

        // Instruction format strings
        for instr in Instruction::iter() {
            let _ = ctx.builder.build_global_string(
                &(instr.to_llvm_name().to_owned() + "\n"),
                &(instr.to_llvm_name().to_owned() + "_fmt"),
            );
        }

        // Error and debug strings
        let _ = ctx.builder.build_global_string(
            "\nStack memory exhausted, terminating program.",
            "exhausted_fmt",
        );
        let _ = ctx
            .builder
            .build_global_string("Calling retry", "retry_fmt");
        let _ = ctx.builder.build_global_string("\n", "newline");
    }
}
