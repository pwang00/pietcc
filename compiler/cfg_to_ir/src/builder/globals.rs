use crate::{builder::build_literals, consts::STACK_SIZE, lowering_ctx::LoweringCtx};
use inkwell::{module::Linkage, AddressSpace};
use piet_core::{instruction::Instruction, state::ExecutionState};
use strum::IntoEnumIterator;

pub(crate) fn build_globals(ctx: &LoweringCtx) {
    build_global_definitions(ctx);
    build_literals(ctx);
}

pub(crate) fn build_global_definitions<'a, 'b>(ctx: &LoweringCtx) {
    let ptr_type = ctx.llvm_context.ptr_type(AddressSpace::default());
    let i8_type = ctx.llvm_context.i8_type();
    let i32_type = ctx.llvm_context.i32_type();
    let i64_type = ctx.llvm_context.i64_type();
    let void_type = ctx.llvm_context.void_type();

    // Global variables
    let piet_stack = ctx.module.add_global(ptr_type, None, "piet_stack");
    piet_stack.set_linkage(Linkage::Internal);
    piet_stack.set_initializer(&ptr_type.const_null());

    let global_stack_size = ctx.module.add_global(i64_type, None, "stack_size");
    global_stack_size.set_linkage(Linkage::Internal);
    global_stack_size.set_initializer(&i64_type.const_zero());

    // Retry counter
    let global_retries = ctx.module.add_global(i8_type, None, "rctr");
    global_retries.set_linkage(Linkage::Internal);
    global_retries.set_initializer(&i8_type.const_zero());

    // extern C io
    let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
    ctx.module.add_function("printf", printf_type, None);

    let scanf_type = i32_type.fn_type(&[ptr_type.into()], true);
    ctx.module.add_function("__isoc99_scanf", scanf_type, None);

    let exit_type = void_type.fn_type(&[i32_type.into()], false);
    ctx.module.add_function("exit", exit_type, None);

    let fdopen_type = ptr_type.fn_type(&[i32_type.into(), ptr_type.into()], false);
    ctx.module.add_function("fdopen", fdopen_type, None);

    let setvbuf_type = i32_type.fn_type(
        &[
            ptr_type.into(),
            ptr_type.into(),
            i32_type.into(),
            i64_type.into(),
        ],
        false,
    );
    ctx.module.add_function("setvbuf", setvbuf_type, None);

    // LLVM intrinsics for roll
    let stack_save_type = ptr_type.fn_type(&[], false);
    ctx.module
        .add_function("llvm.stacksave", stack_save_type, None);

    let stack_restore_type = void_type.fn_type(&[ptr_type.into()], false);
    ctx.module
        .add_function("llvm.stackrestore", stack_restore_type, None);

    let smax_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    ctx.module.add_function("llvm.smax.i64", smax_type, None);

    // Main functions
    let main_fn_type = i64_type.fn_type(&[], false);
    ctx.module.add_function("main", main_fn_type, None);

    let start_fn_type = void_type.fn_type(&[], false);
    ctx.module.add_function("start", start_fn_type, None);

    // Utility functions
    let init_fn_type = void_type.fn_type(&[], false);
    ctx.module.add_function("init_globals", init_fn_type, None);

    let print_stack_fn_type = void_type.fn_type(&[], false);
    ctx.module
        .add_function("print_piet_stack", print_stack_fn_type, None);

    let set_stdout_unbuffered_fn_type = void_type.fn_type(&[], false);
    ctx.module
        .add_function("set_stdout_unbuffered", set_stdout_unbuffered_fn_type, None);

    let terminate_fn_type = i64_type.fn_type(&[], false);
    ctx.module
        .add_function("terminate", terminate_fn_type, None);

    let stack_size_check_fn_type = void_type.fn_type(&[], false);
    ctx.module
        .add_function("stack_size_check", stack_size_check_fn_type, None);

    // Pointer manipulation functions
    let void_fn_type = void_type.fn_type(&[], false);
    ctx.module
        .add_function(Instruction::Swi.to_llvm_name(), void_fn_type, None);
    ctx.module.add_function("piet_rotate", void_fn_type, None);
    ctx.module.add_function("retry", void_fn_type, None);

    // Stack manipulation functions
    ctx.module
        .add_function(Instruction::Dup.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Pop.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Not.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Roll.to_llvm_name(), void_fn_type, None);

    let push_fn_type = void_type.fn_type(&[i64_type.into()], false);
    ctx.module
        .add_function(Instruction::Push.to_llvm_name(), push_fn_type, None);

    // Binary operation functions
    ctx.module
        .add_function(Instruction::Add.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Sub.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Mul.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Div.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Mod.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::Gt.to_llvm_name(), void_fn_type, None);

    // I/O functions
    ctx.module
        .add_function(Instruction::IntIn.to_llvm_name(), void_fn_type, None);
    ctx.module
        .add_function(Instruction::CharIn.to_llvm_name(), void_fn_type, None);
    ctx.module.add_function("piet_intout", void_fn_type, None);
    ctx.module.add_function("piet_charout", void_fn_type, None);
}

pub(crate) fn build_dp_cc<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, execution_state: &ExecutionState) {
    let i8_type = ctx.llvm_context.i8_type();

    let init_dp = i8_type.const_int(execution_state.pointers.dp as u64, false);
    let init_cc = i8_type.const_int(execution_state.pointers.cc as u64, false);

    let global_dp = ctx.module.add_global(i8_type, None, "dp");
    let global_cc = ctx.module.add_global(i8_type, None, "cc");

    global_dp.set_linkage(Linkage::Internal);
    global_dp.set_initializer(&init_dp);

    global_cc.set_linkage(Linkage::Internal);
    global_cc.set_initializer(&init_cc);
}
