use piet_core::instruction::Instruction;

use crate::lowering_ctx::LoweringCtx;

#[allow(unused)]
pub(crate) fn build_push<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let void_type = ctx.llvm_context.void_type();
    let push_fn_type = void_type.fn_type(&[ctx.llvm_context.i64_type().into()], false);
    let push_fn = ctx
        .module
        .add_function(Instruction::Push.to_llvm_name(), push_fn_type, None);

    let basic_block = ctx.llvm_context.append_basic_block(push_fn, "");
    ctx.builder.position_at_end(basic_block);

    let stack_size_check_fn = ctx.module.get_function("stack_size_check").unwrap();
    ctx.builder
        .build_call(stack_size_check_fn, &[], "call_stack_size_check");
    let stack_addr = ctx
        .module
        .get_global("piet_stack")
        .unwrap()
        .as_pointer_value();

    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();

    let stack_size_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "stack_size")
        .unwrap()
        .into_int_value();

    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);

    let load_piet_stack = ctx
        .builder
        .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
        .unwrap()
        .into_pointer_value();

    let top_ptr = unsafe {
        ctx.builder
            .build_gep(
                ctx.llvm_context.i64_type(),
                load_piet_stack,
                &[stack_size_val],
                "top_elem_ptr",
            )
            .unwrap()
    };

    let top_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr, "top_elem_val");

    let first_param = push_fn.get_first_param().unwrap();
    ctx.builder.build_store(top_ptr, first_param);

    let updated_stack_size = ctx
        .builder
        .build_int_add(stack_size_val, const_1, "increment_stack_size")
        .unwrap();

    ctx.builder.build_store(stack_size_addr, updated_stack_size);

    ctx.builder.build_return(None);
}
