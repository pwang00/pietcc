use crate::lowering_ctx::LoweringCtx;
use inkwell::IntPredicate;
use piet_core::instruction::Instruction;
pub(crate) fn build_switch<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let switch_fn = ctx.module.add_function(
        Instruction::Swi.to_llvm_name(),
        ctx.llvm_context.void_type().fn_type(&[], false),
        None,
    );

    let const_0 = ctx.llvm_context.i64_type().const_int(0, false);
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);
    let const_2 = ctx.llvm_context.i64_type().const_int(2, false);
    let const_2_i8 = ctx.llvm_context.i8_type().const_int(2, false);
    let basic_block = ctx.llvm_context.append_basic_block(switch_fn, "");
    ctx.builder.position_at_end(basic_block);
    let then_block = ctx.llvm_context.append_basic_block(switch_fn, "stack_nonempty");
    let lz_block = ctx.llvm_context.append_basic_block(switch_fn, "mod_lz");
    let gz_block = ctx.llvm_context.append_basic_block(switch_fn, "mod_gz");
    let ret_block = ctx.llvm_context.insert_basic_block_after(gz_block, "ret");

    let cc_addr = ctx.module.get_global("cc").unwrap().as_pointer_value();
    let cc_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), cc_addr, "")
        .unwrap()
        .into_int_value();

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

    let stack_size_cmp = ctx
        .builder
        .build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        )
        .unwrap();

    let _ = ctx
        .builder
        .build_conditional_branch(stack_size_cmp, then_block, ret_block);
    ctx.builder.position_at_end(then_block);

    let top_idx = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "top_elem_idx")
        .unwrap();

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
    let top_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr_gep, "top_elem_val")
        .unwrap()
        .into_int_value();

    let res = ctx
        .builder
        .build_int_signed_rem(top_ptr_val, const_2, "mod_by_2")
        .unwrap();

    // If rem < 0 then we store -rem, otherwise we store rem
    let cmp = ctx
        .builder
        .build_int_compare(IntPredicate::SGE, res, const_0, "cmp_mod_gz")
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(cmp, lz_block, gz_block);
    ctx.builder.position_at_end(lz_block);
    let decremented = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
        .unwrap();
    let _ = ctx.builder.build_store(stack_size_addr, decremented);
    let rem_lz = ctx.builder.build_int_neg(res, "neg_res").unwrap();
    let rem_lz = ctx
        .builder
        .build_int_truncate(rem_lz, ctx.llvm_context.i8_type(), "trunc_to_i8")
        .unwrap();
    let rem_lz = ctx.builder.build_int_add(rem_lz, cc_val, "").unwrap();
    let rem_lz = ctx
        .builder
        .build_int_unsigned_rem(rem_lz, const_2_i8, "")
        .unwrap();
    let _ = ctx.builder.build_store(cc_addr, rem_lz);
    let _ = ctx.builder.build_unconditional_branch(ret_block);

    ctx.builder.position_at_end(gz_block);
    let decremented = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
        .unwrap();
    ctx.builder
        .build_store(stack_size_addr, decremented)
        .unwrap();
    let rem_gz = ctx
        .builder
        .build_int_truncate(res, ctx.llvm_context.i8_type(), "trunc_to_i8")
        .unwrap();
    let rem_gz = ctx.builder.build_int_add(rem_gz, cc_val, "").unwrap();
    let rem_gz = ctx
        .builder
        .build_int_unsigned_rem(rem_gz, const_2_i8, "")
        .unwrap();
    let _ = ctx.builder.build_store(cc_addr, rem_gz);
    let _ = ctx.builder.build_unconditional_branch(ret_block);
    // Return
    ctx.builder.position_at_end(ret_block);
    let _ = ctx.builder.build_return(None);
}
pub(crate) fn build_rotate<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let rotate_fn_type = ctx.llvm_context.void_type().fn_type(&[], false);
    let rotate_fn = ctx.module.add_function("piet_rotate", rotate_fn_type, None);

    let const_0 = ctx.llvm_context.i64_type().const_int(0, false);
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);
    let const_4 = ctx.llvm_context.i64_type().const_int(4, false);
    let const_4_i8 = ctx.llvm_context.i8_type().const_int(4, false);
    let basic_block = ctx.llvm_context.append_basic_block(rotate_fn, "");
    ctx.builder.position_at_end(basic_block);
    let then_block = ctx.llvm_context.append_basic_block(rotate_fn, "stack_nonempty");

    let lz_block = ctx.llvm_context.append_basic_block(rotate_fn, "top_lz");
    let gz_block = ctx.llvm_context.append_basic_block(rotate_fn, "top_gz");
    let ret_block = ctx.llvm_context.insert_basic_block_after(gz_block, "ret");

    let dp_addr = ctx.module.get_global("dp").unwrap().as_pointer_value();
    let dp_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), dp_addr, "")
        .unwrap()
        .into_int_value();
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

    let stack_size_cmp = ctx
        .builder
        .build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        )
        .unwrap();

    let _ = ctx
        .builder
        .build_conditional_branch(stack_size_cmp, then_block, ret_block);
    ctx.builder.position_at_end(then_block);

    let top_idx = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "top_elem_idx")
        .unwrap();

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

    let top_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr_gep, "top_elem_val")
        .unwrap()
        .into_int_value();

    let rem = ctx
        .builder
        .build_int_signed_rem(top_ptr_val, const_4, "rem")
        .unwrap();

    let cmp = ctx
        .builder
        .build_int_compare(IntPredicate::SGE, top_ptr_val, const_0, "cmp_top_zero")
        .unwrap();

    let _ = ctx
        .builder
        .build_conditional_branch(cmp, gz_block, lz_block);
    ctx.builder.position_at_end(lz_block);
    let decremented = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
        .unwrap();
    ctx.builder
        .build_store(stack_size_addr, decremented)
        .unwrap();
    let rem_lz = ctx
        .builder
        .build_int_add(rem, const_4, "updated_rem")
        .unwrap();
    let rem_lz = ctx
        .builder
        .build_int_truncate(rem_lz, ctx.llvm_context.i8_type(), "trunc_to_i8")
        .unwrap();
    let rem_lz = ctx.builder.build_int_add(rem_lz, dp_val, "").unwrap();
    let rem_lz = ctx
        .builder
        .build_int_unsigned_rem(rem_lz, const_4_i8, "")
        .unwrap();
    let _ = ctx.builder.build_store(dp_addr, rem_lz);
    let _ = ctx.builder.build_unconditional_branch(ret_block);

    ctx.builder.position_at_end(gz_block);
    let decremented = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
        .unwrap();
    let _ = ctx.builder.build_store(stack_size_addr, decremented);
    let rem_gz = ctx
        .builder
        .build_int_truncate(rem, ctx.llvm_context.i8_type(), "trunc_to_i8")
        .unwrap();
    let rem_gz = ctx.builder.build_int_add(rem_gz, dp_val, "").unwrap();
    let rem_gz = ctx
        .builder
        .build_int_unsigned_rem(rem_gz, const_4_i8, "")
        .unwrap();
    let _ = ctx.builder.build_store(dp_addr, rem_gz);
    let _ = ctx.builder.build_unconditional_branch(ret_block);
    // Return
    ctx.builder.position_at_end(ret_block);
    let _ = ctx.builder.build_return(None);
}
pub(crate) fn build_retry<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    let void_type = ctx.llvm_context.void_type();
    let i8_type = ctx.llvm_context.i8_type();
    let retry_fn_type = void_type.fn_type(&[], false);
    let retry_fn = ctx.module.add_function("retry", retry_fn_type, None);
    // Basic blocks
    let basic_block = ctx.llvm_context.append_basic_block(retry_fn, "");
    let one_mod_two = ctx.llvm_context.append_basic_block(retry_fn, "one_mod_two");
    let zero_mod_two = ctx.llvm_context.append_basic_block(retry_fn, "zero_mod_two");
    let ret_block = ctx.llvm_context.append_basic_block(retry_fn, "ret");

    ctx.builder.position_at_end(basic_block);
    // Constants
    let const_1 = i8_type.const_int(1, false);
    let const_2 = i8_type.const_int(2, false);
    let const_4 = i8_type.const_int(4, false);
    let const_8 = i8_type.const_int(8, false);
    // Pointers dp and cc
    let dp_addr = ctx.module.get_global("dp").unwrap().as_pointer_value();
    let cc_addr = ctx.module.get_global("cc").unwrap().as_pointer_value();
    let rctr_addr = ctx.module.get_global("rctr").unwrap().as_pointer_value();

    // Loaded values
    let dp_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), dp_addr, "load_dp")
        .unwrap()
        .into_int_value();
    let cc_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), cc_addr, "load_cc")
        .unwrap()
        .into_int_value();
    let rctr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), rctr_addr, "load_rctr")
        .unwrap()
        .into_int_value();

    let rem = ctx
        .builder
        .build_int_unsigned_rem(rctr_val, const_2, "")
        .unwrap();
    let cmp = ctx
        .builder
        .build_int_compare(IntPredicate::EQ, rem, const_1, "")
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(cmp, one_mod_two, zero_mod_two);

    // One mod two
    ctx.builder.position_at_end(one_mod_two);
    let dp_sum = ctx
        .builder
        .build_int_add(dp_val, const_1, "rotate_dp")
        .unwrap();
    let dp_mod_4 = ctx
        .builder
        .build_int_unsigned_rem(dp_sum, const_4, "dp_mod_4")
        .unwrap();
    let dp_to_i8 = ctx
        .builder
        .build_int_truncate(dp_mod_4, ctx.llvm_context.i8_type(), "trunc_dp_to_i8")
        .unwrap();
    let _ = ctx.builder.build_store(dp_addr, dp_to_i8);
    let _ = ctx.builder.build_unconditional_branch(ret_block);

    // Zero mod two
    ctx.builder.position_at_end(zero_mod_two);
    let cc_sum = ctx
        .builder
        .build_int_add(cc_val, const_1, "rotate_cc")
        .unwrap();
    let cc_mod_2 = ctx
        .builder
        .build_int_unsigned_rem(cc_sum, const_2, "dp_mod_4")
        .unwrap();
    let cc_to_i8 = ctx
        .builder
        .build_int_truncate(cc_mod_2, ctx.llvm_context.i8_type(), "trunc_cc_to_i8")
        .unwrap();
    let _ = ctx.builder.build_store(cc_addr, cc_to_i8);
    let _ = ctx.builder.build_unconditional_branch(ret_block);

    // Ret
    ctx.builder.position_at_end(ret_block);
    //ctx.builder.build_call(print_ptr_fn, &[], "");
    let rctr_added = ctx.builder.build_int_add(rctr_val, const_1, "").unwrap();
    let rctr_mod_8 = ctx
        .builder
        .build_int_unsigned_rem(rctr_added, const_8, "")
        .unwrap();
    let rctr_to_i8 = ctx
        .builder
        .build_int_truncate(rctr_mod_8, ctx.llvm_context.i8_type(), "trunc_to_i8")
        .unwrap();
    let _ = ctx.builder.build_store(rctr_addr, rctr_to_i8);
    let _ = ctx.builder.build_return(None);
}
