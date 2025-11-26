use inkwell::{
    values::{AnyValue, BasicValue, IntValue, PointerValue},
    IntPredicate,
};
use piet_core::instruction::Instruction;

use crate::lowering_ctx::LoweringCtx;
pub(crate) fn build_roll<'a, 'b>(ctx: &LoweringCtx<'a, 'b>) {
    // Roll function type
    let roll_fn = ctx
        .module
        .get_function(Instruction::Roll.to_llvm_name())
        .unwrap();

    // LLVM intrinsics
    let stack_save_fn = ctx.module.get_function("llvm.stacksave").unwrap();
    let stack_restore_fn = ctx.module.get_function("llvm.stackrestore").unwrap();
    let llvm_smax_fn = ctx.module.get_function("llvm.smax.i64").unwrap();

    // Globals
    let stack_size_addr = ctx
        .module
        .get_global("stack_size")
        .unwrap()
        .as_pointer_value();
    let stack_addr = ctx
        .module
        .get_global("piet_stack")
        .unwrap()
        .as_pointer_value();

    // Constants
    let const_neg_1 = ctx.llvm_context.i64_type().const_int(u64::MAX, true);
    let const_0 = ctx.llvm_context.i64_type().const_zero();
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);
    let const_2 = ctx.llvm_context.i64_type().const_int(2, false);

    // Basic blocks
    let entry = ctx.llvm_context.append_basic_block(roll_fn, "");
    ctx.builder.position_at_end(entry);
    let block_3 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_14 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_18 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_22 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_27 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_30 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_32 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_41 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_44 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_45 = ctx.llvm_context.append_basic_block(roll_fn, "");
    let block_55 = ctx.llvm_context.append_basic_block(roll_fn, "");

    // Load stack size
    let stack_size_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    stack_size_load_instr.set_alignment(8).ok();
    let stack_size_val = stack_size_load_instr.try_into().unwrap();

    let cmp = ctx
        .builder
        .build_int_compare(IntPredicate::SLT, stack_size_val, const_2, "")
        .unwrap();
    let _ = ctx.builder.build_conditional_branch(cmp, block_55, block_3);

    // Block 3
    ctx.builder.position_at_end(block_3);
    let load_piet_stack = ctx
        .builder
        .build_load(stack_addr.get_type(), stack_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    load_piet_stack.set_alignment(8).ok();
    let load_piet_stack = load_piet_stack.as_any_value_enum().into_pointer_value();

    let top_idx = ctx
        .builder
        .build_int_nsw_sub(stack_size_val, const_1, "")
        .unwrap();
    let top_elem_addr = unsafe {
        ctx.builder
            .build_in_bounds_gep(load_piet_stack.get_type(), load_piet_stack, &[top_idx], "")
            .unwrap()
    };
    let top_elem_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_elem_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    top_elem_load_instr.set_alignment(8).ok();
    let top_elem_val = top_elem_load_instr.as_any_value_enum().into_int_value();

    let size_minus_two = ctx
        .builder
        .build_int_nsw_sub(stack_size_val, const_2, "")
        .unwrap();
    let next_elem_addr = unsafe {
        ctx.builder
            .build_in_bounds_gep(
                load_piet_stack.get_type(),
                load_piet_stack,
                &[size_minus_two],
                "",
            )
            .unwrap()
    };
    let next_elem_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), next_elem_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    next_elem_load_instr.set_alignment(8).ok();
    let next_elem_val = next_elem_load_instr.try_into().unwrap();

    let store = ctx
        .builder
        .build_store(stack_size_addr, size_minus_two)
        .unwrap();
    store.set_alignment(8).ok();

    let left = ctx
        .builder
        .build_int_compare(IntPredicate::SLT, size_minus_two, next_elem_val, "")
        .unwrap();
    let right = ctx
        .builder
        .build_int_compare(IntPredicate::SLT, next_elem_val, const_0, "")
        .unwrap();
    let or = ctx.builder.build_or(left, right, "").unwrap();
    let _ = ctx.builder.build_conditional_branch(or, block_55, block_14);

    // Block 14
    ctx.builder.position_at_end(block_14);
    let stack_save = ctx.builder.build_call(stack_save_fn, &[], "").unwrap();
    stack_save.set_tail_call(true);
    let stack_alloc = ctx
        .builder
        .build_array_alloca(ctx.llvm_context.i64_type(), next_elem_val, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    stack_alloc.set_alignment(16).ok();
    let rolls_ltz = ctx
        .builder
        .build_int_compare(IntPredicate::SLT, top_elem_val, const_0, "")
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(rolls_ltz, block_18, block_22);

    // Block 18
    ctx.builder.position_at_end(block_18);
    let sub1 = ctx
        .builder
        .build_int_sub(const_0, top_elem_val, "")
        .unwrap();
    let rem = ctx
        .builder
        .build_int_unsigned_rem(sub1, next_elem_val, "")
        .unwrap();
    let sub2 = ctx
        .builder
        .build_int_nsw_sub(next_elem_val, rem, "")
        .unwrap();
    let _ = ctx.builder.build_unconditional_branch(block_22);

    // Block 22
    ctx.builder.position_at_end(block_22);
    let phi1 = ctx
        .builder
        .build_phi(ctx.llvm_context.i64_type(), "")
        .unwrap();
    phi1.add_incoming(&[
        (&sub2.as_basic_value_enum(), block_18),
        (&top_elem_val.as_basic_value_enum(), block_14),
    ]);

    let add = ctx
        .builder
        .build_int_nsw_add(phi1.as_basic_value().into_int_value(), next_elem_val, "")
        .unwrap();

    let load_piet_stack2 = ctx
        .builder
        .build_load(stack_addr.get_type(), stack_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    load_piet_stack2.set_alignment(8).ok();
    let load_piet_stack2: PointerValue = load_piet_stack2.try_into().unwrap();

    let stack_size_load_instr2 = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    stack_size_load_instr2.set_alignment(8).ok();
    let stack_size_val2: IntValue = stack_size_load_instr2.try_into().unwrap();
    let _ = ctx.builder.build_unconditional_branch(block_27);

    // Block 27
    ctx.builder.position_at_end(block_27);
    let phi2 = ctx
        .builder
        .build_phi(ctx.llvm_context.i64_type(), "")
        .unwrap();
    phi2.add_incoming(&[(&phi1.as_basic_value(), block_22)]);
    let cmp3 = ctx
        .builder
        .build_int_compare(
            IntPredicate::SLT,
            phi2.as_basic_value().into_int_value(),
            add,
            "",
        )
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(cmp3, block_32, block_30);

    // Block 30
    ctx.builder.position_at_end(block_30);
    let call2 = ctx
        .builder
        .build_call(llvm_smax_fn, &[next_elem_val.into(), const_0.into()], "")
        .unwrap();
    let _ = ctx.builder.build_unconditional_branch(block_41);

    // Block 32
    ctx.builder.position_at_end(block_32);
    let rem2 = ctx
        .builder
        .build_int_signed_rem(phi2.as_basic_value().into_int_value(), next_elem_val, "")
        .unwrap();
    let sign_change = ctx.builder.build_xor(rem2, const_neg_1, "").unwrap();
    let make_positive = ctx
        .builder
        .build_int_add(stack_size_val2, sign_change, "")
        .unwrap();
    let curr_ptr = unsafe {
        ctx.builder
            .build_in_bounds_gep(
                load_piet_stack2.get_type(),
                load_piet_stack2,
                &[make_positive],
                "",
            )
            .unwrap()
    };
    let curr_ptr_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), curr_ptr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    curr_ptr_load_instr.set_alignment(8).ok();
    let curr_val = curr_ptr_load_instr.as_any_value_enum().into_int_value();
    let sub_phis = ctx
        .builder
        .build_int_nsw_sub(
            phi2.as_basic_value().into_int_value(),
            phi1.as_basic_value().into_int_value(),
            "",
        )
        .unwrap();
    let slice_ptr = unsafe {
        ctx.builder
            .build_in_bounds_gep(
                stack_alloc
                    .as_any_value_enum()
                    .into_pointer_value()
                    .get_type(),
                stack_alloc.as_any_value_enum().into_pointer_value(),
                &[sub_phis],
                "",
            )
            .unwrap()
    };
    let store_in_slice = ctx.builder.build_store(slice_ptr, curr_val).unwrap();
    store_in_slice.set_alignment(8).ok();
    let incr_idx = ctx
        .builder
        .build_int_nsw_add(phi2.as_basic_value().into_int_value(), const_1, "")
        .unwrap();
    phi2.add_incoming(&[(&incr_idx.as_basic_value_enum(), block_32)]);
    let _ = ctx.builder.build_unconditional_branch(block_27);

    // Block 41
    ctx.builder.position_at_end(block_41);
    let phi3 = ctx
        .builder
        .build_phi(ctx.llvm_context.i64_type(), "")
        .unwrap();
    let cmp3 = ctx
        .builder
        .build_int_compare(
            IntPredicate::EQ,
            phi3.as_basic_value().into_int_value(),
            call2.as_any_value_enum().into_int_value(),
            "",
        )
        .unwrap();
    let _ = ctx
        .builder
        .build_conditional_branch(cmp3, block_44, block_45);

    // Block 44
    ctx.builder.position_at_end(block_44);
    let restore = ctx
        .builder
        .build_call(
            stack_restore_fn,
            &[stack_save.try_as_basic_value().unwrap_basic().into()],
            "",
        )
        .unwrap();
    restore.set_tail_call(true);
    let _ = ctx.builder.build_unconditional_branch(block_55);

    // Block 45
    ctx.builder.position_at_end(block_45);
    let sign_change = ctx
        .builder
        .build_xor(phi3.as_basic_value().into_int_value(), const_neg_1, "")
        .unwrap();
    let add = ctx
        .builder
        .build_int_add(next_elem_val, sign_change, "")
        .unwrap();
    let slice_ptr = unsafe {
        ctx.builder
            .build_in_bounds_gep(
                stack_alloc
                    .as_any_value_enum()
                    .into_pointer_value()
                    .get_type(),
                stack_alloc.as_any_value_enum().into_pointer_value(),
                &[add],
                "",
            )
            .unwrap()
    };
    let slice_elem_load_instr = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), slice_ptr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    slice_elem_load_instr.set_alignment(8).ok();
    let slice_elem = slice_elem_load_instr.as_any_value_enum().into_int_value();

    let stack_size_load_instr3 = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), stack_size_addr, "")
        .unwrap()
        .as_instruction_value()
        .unwrap();
    stack_size_load_instr3.set_alignment(8).ok();
    let stack_size_val = stack_size_load_instr3.as_any_value_enum().into_int_value();

    let sub_idx = ctx
        .builder
        .build_int_sub(phi3.as_basic_value().into_int_value(), next_elem_val, "")
        .unwrap();
    let add_idx = ctx
        .builder
        .build_int_add(sub_idx, stack_size_val, "")
        .unwrap();

    let curr_stack_ptr = unsafe {
        ctx.builder
            .build_in_bounds_gep(
                load_piet_stack2.get_type(),
                load_piet_stack2,
                &[add_idx],
                "",
            )
            .unwrap()
    };
    let store_in_stack = ctx.builder.build_store(curr_stack_ptr, slice_elem).unwrap();
    store_in_stack.set_alignment(8).ok();
    let incr_idx = ctx
        .builder
        .build_int_nuw_add(phi3.as_basic_value().into_int_value(), const_1, "")
        .unwrap();
    phi3.add_incoming(&[
        (&incr_idx.as_basic_value_enum(), block_45),
        (&const_0, block_30),
    ]);
    let _ = ctx.builder.build_unconditional_branch(block_41);
    // Return
    ctx.builder.position_at_end(block_55);
    let _ = ctx.builder.build_return(None);
}
