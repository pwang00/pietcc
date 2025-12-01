use inkwell::AddressSpace;
use piet_core::state::ExecutionState;

use crate::lowering_ctx::LoweringCtx;

pub(crate) fn build_stack_io(ctx: &LoweringCtx, execution_state: &ExecutionState) {
    let i64_type = ctx.llvm_context.i64_type();
    let printf_fn = ctx.module.get_function("printf").unwrap();
    let initialize_piet_stack_fn = ctx.module.get_function("initialize_piet_stack").unwrap();
    let initialize_piet_stack_body = ctx
        .llvm_context
        .append_basic_block(initialize_piet_stack_fn, "initialize_piet_stack_body");

    ctx.builder.position_at_end(initialize_piet_stack_body);

    let curr_stack_size = ctx
        .llvm_context
        .i64_type()
        .const_int(execution_state.stack.len() as u64, false);
    let stack_size_val = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();
    ctx.builder
        .build_store(stack_size_val, curr_stack_size)
        .unwrap();

    let stdout: String = execution_state
        .stdout
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("");

    let stdout_state = unsafe {
        ctx.builder
            .build_global_string(&stdout, "stdout_state")
            .unwrap()
            .as_pointer_value()
    };

    ctx.builder
        .build_call(printf_fn, &[stdout_state.into()], "call_printf_stdout")
        .unwrap();

    let stack_addr = ctx
        .module
        .get_global("piet_stack")
        .unwrap()
        .as_pointer_value();

    let piet_stack = ctx
        .builder
        .build_load(
            ctx.llvm_context.ptr_type(AddressSpace::default()),
            stack_addr,
            "piet_stack_load",
        )
        .unwrap()
        .into_pointer_value();

    for (idx, &val) in execution_state.stack.iter().rev().enumerate() {
        let llvm_idx = ctx.llvm_context.i64_type().const_int(idx as u64, false);
        unsafe {
            let offset = ctx
                .builder
                .build_gep(i64_type, piet_stack, &[llvm_idx], "index")
                .unwrap();
            let stack_val = ctx.llvm_context.i64_type().const_int(val as u64, false);
            ctx.builder.build_store(offset, stack_val).unwrap();
        }
    }
    ctx.builder.build_return(None).unwrap();
}
