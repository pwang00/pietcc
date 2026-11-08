use crate::lowering_ctx::LoweringCtx;
use inkwell::IntPredicate;
use piet_core::instruction::Instruction;

pub(crate) fn build_binops<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, instr: Instruction) {
    let binop_fn = match instr {
        Instruction::Add
        | Instruction::Sub
        | Instruction::Div
        | Instruction::Mul
        | Instruction::Mod
        | Instruction::Gt => ctx.module.get_function(instr.to_llvm_name()).unwrap(),
        _ => panic!("Not a binary operation!"),
    };

    // i64s are 64 bits, so we want to do stack[stack_size - 1] + stack[stack_size - 2] if possible
    // Basic blocks (Only some will be used depending on the operation)
    let basic_block = ctx.llvm_context.append_basic_block(binop_fn, "");
    ctx.builder.position_at_end(basic_block);
    let cont_block = ctx.llvm_context.append_basic_block(binop_fn, "cont");
    let dividend_nonzero = ctx
        .llvm_context
        .append_basic_block(binop_fn, "dividend_nonzero");
    let then_block = ctx.llvm_context.append_basic_block(binop_fn, "");
    let else_block = ctx.llvm_context.append_basic_block(binop_fn, "");
    let ret_block = ctx.llvm_context.insert_basic_block_after(else_block, "ret");

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

    let const_0 = ctx.llvm_context.i64_type().const_zero();
    let const_1 = ctx.llvm_context.i64_type().const_int(1, false);
    let const_2 = ctx.llvm_context.i64_type().const_int(2, false);

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

    let cmp = ctx
        .builder
        .build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_2,
            "check_stack_size",
        )
        .unwrap();

    ctx.builder
        .build_conditional_branch(cmp, cont_block, ret_block)
        .unwrap();

    // Enough elems on stack
    ctx.builder.position_at_end(cont_block);

    let top_idx = ctx
        .builder
        .build_int_sub(stack_size_val, const_1, "top_elem_idx");

    let top_ptr_gep = unsafe {
        ctx.builder
            .build_gep(
                ctx.llvm_context.i64_type(),
                load_piet_stack,
                &[top_idx.unwrap()],
                "",
            )
            .unwrap()
    };

    let next_idx = ctx
        .builder
        .build_int_sub(stack_size_val, const_2, "next_elem_idx")
        .unwrap();

    let next_ptr = unsafe {
        ctx.builder
            .build_gep(
                ctx.llvm_context.i64_type(),
                load_piet_stack,
                &[next_idx],
                "",
            )
            .unwrap()
    };

    let top_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), top_ptr_gep, "top_elem_val")
        .unwrap()
        .into_int_value();

    let next_ptr_val = ctx
        .builder
        .build_load(ctx.llvm_context.i64_type(), next_ptr, "next_elem_val")
        .unwrap()
        .into_int_value();

    let result = match instr {
        Instruction::Add => {
            unsafe { then_block.delete().ok() };
            unsafe { else_block.delete().ok() };
            unsafe { dividend_nonzero.delete().ok() };
            ctx.builder
                .build_int_add(next_ptr_val, top_ptr_val, "add")
                .unwrap()
        }
        Instruction::Sub => {
            unsafe { then_block.delete().ok() };
            unsafe { else_block.delete().ok() };
            unsafe { dividend_nonzero.delete().ok() };
            ctx.builder
                .build_int_sub(next_ptr_val, top_ptr_val, "sub")
                .unwrap()
        }
        Instruction::Mul => {
            unsafe { then_block.delete().ok() };
            unsafe { else_block.delete().ok() };
            unsafe { dividend_nonzero.delete().ok() };
            ctx.builder
                .build_int_mul(next_ptr_val, top_ptr_val, "mul")
                .unwrap()
        }
        Instruction::Div => {
            let cmp = ctx.builder.build_int_compare(
                IntPredicate::NE,
                top_ptr_val,
                const_0,
                "check_dividend_nonzero",
            );
            ctx.builder
                .build_conditional_branch(cmp.unwrap(), dividend_nonzero, ret_block)
                .unwrap();

            // Set names for blocks and delete unused
            unsafe { then_block.delete().ok() };
            unsafe { else_block.delete().ok() };

            ctx.builder.position_at_end(dividend_nonzero);
            ctx.builder
                .build_int_signed_div(next_ptr_val, top_ptr_val, "div")
                .unwrap()
        }
        Instruction::Mod => {
            let i64_type = ctx.llvm_context.i64_type();

            // Calculate the absolute value without a branch
            let shift_amount = i64_type.const_int(63, false);
            let a_asr = ctx
                .builder
                .build_right_shift(top_ptr_val, shift_amount, true, "arithmetic_right_shift")
                .unwrap();
            let a_xor = ctx.builder.build_xor(top_ptr_val, a_asr, "a_xor");
            let top_ptr_val = ctx
                .builder
                .build_int_sub(a_xor.unwrap(), a_asr, "abs_a")
                .unwrap();

            let cmp = ctx.builder.build_int_compare(
                IntPredicate::NE,
                top_ptr_val,
                const_0,
                "check_dividend_nonzero",
            );

            ctx.builder
                .build_conditional_branch(cmp.unwrap(), dividend_nonzero, ret_block)
                .unwrap();

            ctx.builder.position_at_end(dividend_nonzero);
            let rem = ctx
                .builder
                .build_int_signed_rem(next_ptr_val, top_ptr_val, "mod")
                .unwrap();

            let store_rem_result = ctx
                .builder
                .build_alloca(ctx.llvm_context.i64_type(), "rem_result")
                .unwrap();
            ctx.builder.build_store(store_rem_result, rem).unwrap();

            let cmp =
                ctx.builder
                    .build_int_compare(IntPredicate::SGE, rem, const_0, "check_mod_sign");

            then_block.set_name("lz");
            else_block.set_name("gez");

            ctx.builder
                .build_conditional_branch(cmp.unwrap(), else_block, then_block)
                .unwrap();

            ctx.builder.position_at_end(then_block);
            let rem_lz = ctx.builder.build_int_add(top_ptr_val, rem, "rem_lz");
            ctx.builder
                .build_store(store_rem_result, rem_lz.unwrap())
                .unwrap();
            ctx.builder.build_unconditional_branch(else_block).unwrap();
            ctx.builder.position_at_end(else_block);
            ctx.builder
                .build_load(ctx.llvm_context.i64_type(), store_rem_result, "load_result")
                .unwrap()
                .into_int_value()
        }
        Instruction::Gt => {
            // Pushes 1 to stack if second top > top otherwise 0
            unsafe { then_block.delete().ok() };
            unsafe { else_block.delete().ok() };
            unsafe { dividend_nonzero.delete().ok() };

            let diff = ctx.builder.build_int_sub(next_ptr_val, top_ptr_val, "sub");
            let value_cmp = ctx.builder.build_int_compare(
                IntPredicate::SGT,
                diff.unwrap(),
                const_0,
                "check_next_gt_top",
            );
            let res = ctx.builder.build_int_z_extend(
                value_cmp.unwrap(),
                ctx.llvm_context.i64_type(),
                "zero_extend_cmp",
            );

            res.unwrap()
        }
        _ => panic!("Not a binary operation!"),
    };

    let updated_stack_size =
        ctx.builder
            .build_int_sub(stack_size_val, const_1, "decrement_stack_size");

    ctx.builder
        .build_store(stack_size_addr, updated_stack_size.unwrap())
        .unwrap();

    ctx.builder.build_store(next_ptr, result).unwrap();
    ctx.builder.build_unconditional_branch(ret_block).unwrap();
    // Not enough elems on stack
    ctx.builder.position_at_end(ret_block);
    ctx.builder.build_return(None).unwrap();
}
