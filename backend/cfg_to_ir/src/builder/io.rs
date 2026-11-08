use crate::lowering_ctx::LoweringCtx;
use inkwell::{
    values::{AnyValue, BasicValue, IntValue},
    IntPredicate,
};
use piet_core::instruction::Instruction;

#[allow(unused)]
pub(crate) fn build_input<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, instr: Instruction) {
    let in_fn = match instr {
        Instruction::IntIn => ctx
            .module
            .get_function(Instruction::IntIn.to_llvm_name())
            .unwrap(),
        Instruction::CharIn => ctx
            .module
            .get_function(Instruction::CharIn.to_llvm_name())
            .unwrap(),
        _ => panic!("Not an input instruction!"),
    };

    let printf_fn = ctx.module.get_function("printf").unwrap();

    // Consts
    let const_0 = ctx.llvm_context.i64_type().const_zero();
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    // Labels
    let basic_block = ctx.llvm_context.append_basic_block(in_fn, "");
    ctx.builder.position_at_end(basic_block);

    // Local variable to store our input
    let read_addr = ctx
        .builder
        .build_alloca(ctx.llvm_context.i64_type(), "stack_alloc")
        .unwrap();

    // The stack may not necessarily be zero'd out, which may cause problems when printing
    // Since %c only reads in at one-byte boundaries, if the higher bits of our value are nonzero
    // Then printf("%c", val) could print garbage and we would like this not to happen
    ctx.builder.build_store(read_addr, const_0).unwrap();

    let fmt = match instr {
        Instruction::IntIn => ctx.module.get_global("dec_fmt").unwrap().as_pointer_value(),
        Instruction::CharIn => ctx
            .module
            .get_global("char_fmt")
            .unwrap()
            .as_pointer_value(),
        _ => panic!("Not an input instruction"),
    };

    let input_message_fmt = match instr {
        Instruction::IntIn => ctx
            .module
            .get_global("input_message_int")
            .unwrap()
            .as_pointer_value(),
        Instruction::CharIn => ctx
            .module
            .get_global("input_message_char")
            .unwrap()
            .as_pointer_value(),
        _ => panic!("Not an input instruction"),
    };

    // Enter int vs char
    let input_message_fmt_gep = unsafe {
        ctx.builder
            .build_gep(
                input_message_fmt.get_type(),
                input_message_fmt,
                &[const_0, const_0],
                "",
            )
            .unwrap()
    };
    ctx.builder
        .build_call(printf_fn, &[input_message_fmt_gep.into()], "")
        .unwrap();

    // %ld or %c
    let const_fmt_gep = unsafe {
        ctx.builder
            .build_gep(fmt.get_type(), fmt, &[const_0, const_0], "")
            .unwrap()
    };

    // Build scanf call
    // scanf reads into our local variable, so we need to load it next
    let scanf_fn = ctx.module.get_function("scanf").unwrap();
    let scanf = ctx
        .builder
        .build_call(scanf_fn, &[const_fmt_gep.into(), read_addr.into()], "scanf")
        .unwrap();

    // Loads local var and sets alignment
    let load_scanf_elem = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), read_addr, "scanf_elem")
        .unwrap()
        .as_instruction_value()
        .unwrap();

    load_scanf_elem.set_alignment(8);

    let x: IntValue = load_scanf_elem.try_into().unwrap();
    let result = ctx
        .builder
        .build_int_s_extend(x, ctx.llvm_context.i64_type(), "sext_to_i64")
        .unwrap();

    // &stack_size
    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let stack_size_load_instr = ctx
        .builder
        .build_load(
            ctx.llvm_context.i64_type(),
            stack_size_addr,
            "load_stack_size",
        )
        .unwrap()
        .as_instruction_value()
        .unwrap();

    // For some reason Inkwell aligns i64s at a 4 byte boundary and not 8 byte, very weirdga
    stack_size_load_instr.set_alignment(8);
    let stack_size_val: IntValue = stack_size_load_instr.try_into().unwrap();

    // &piet_stack
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

    // Push to stack
    let push_ptr_gep = unsafe {
        ctx.builder
            .build_gep(
                load_piet_stack.get_type(),
                load_piet_stack,
                &[stack_size_val],
                "top_elem_addr",
            )
            .unwrap()
    };

    let store_to_stack = ctx.builder.build_store(push_ptr_gep, result).unwrap();

    store_to_stack.set_alignment(8);

    let updated_stack_size = ctx
        .builder
        .build_int_add(stack_size_val, const_1, "increment_stack_size")
        .unwrap();

    // Store updated stack size
    let store = ctx
        .builder
        .build_store(stack_size_addr, updated_stack_size)
        .unwrap();

    store.set_alignment(8).ok();
    ctx.builder.build_return(None).unwrap();
}
pub(crate) fn build_output<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, instr: Instruction) {
    let out_fn = match instr {
        Instruction::IntOut => ctx.module.get_function("piet_intout").unwrap(),
        Instruction::CharOut => ctx.module.get_function("piet_charout").unwrap(),
        _ => panic!("Not an output instruction!"),
    };

    // Labels
    let basic_block = ctx.llvm_context.append_basic_block(out_fn, "");
    let then_block = ctx
        .llvm_context
        .append_basic_block(out_fn, "stack_nonempty");
    let ret_block = ctx.llvm_context.append_basic_block(out_fn, "ret");

    ctx.builder.position_at_end(basic_block);

    // Constants
    let const_0 = ctx.llvm_context.i64_type().const_zero();
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let stack_size_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "stack_size")
        .unwrap()
        .as_instruction_value()
        .unwrap();

    // For some reason Inkwell aligns i64s at a 4 byte boundary and not 8 byte, very weirdga
    stack_size_load_instr.set_alignment(8).unwrap();

    let stack_size_val = stack_size_load_instr.try_into().unwrap();

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
        .build_conditional_branch(stack_size_cmp, then_block, ret_block)
        .unwrap();
    ctx.builder.position_at_end(then_block);

    let top_idx = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "top_elem_idx")
        .unwrap();

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

    let top_ptr_gep = unsafe {
        ctx.builder
            .build_gep(ctx.llvm_context.i64_type(), load_piet_stack, &[top_idx], "")
            .unwrap()
    };
    let printf_fn = ctx.module.get_function("printf").unwrap();

    let fmt = match instr {
        Instruction::IntOut => ctx.module.get_global("dec_fmt").unwrap().as_pointer_value(),
        Instruction::CharOut => ctx
            .module
            .get_global("char_fmt")
            .unwrap()
            .as_pointer_value(),
        _ => panic!("Not an output instruction"),
    };

    let top_ptr_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr_gep, "top_elem_val")
        .unwrap()
        .as_instruction_value()
        .unwrap();

    top_ptr_load_instr.set_alignment(8).unwrap();

    let top_ptr_val = top_ptr_load_instr.as_any_value_enum().into_int_value();

    let const_fmt_gep = unsafe {
        ctx.builder
            .build_gep(fmt.get_type(), fmt, &[const_0, const_0], "")
            .unwrap()
    };

    let _printf = ctx.builder.build_call(
        printf_fn,
        &[const_fmt_gep.into(), top_ptr_val.into()],
        "printf",
    );

    let updated_stack_size = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
        .unwrap();

    let store = ctx
        .builder
        .build_store(stack_size_addr, updated_stack_size)
        .unwrap();

    store.set_alignment(8).ok();
    ctx.builder.build_unconditional_branch(ret_block).unwrap();

    ctx.builder.position_at_end(ret_block);
    ctx.builder.build_return(None).unwrap();
}
