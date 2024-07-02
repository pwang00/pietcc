use inkwell::IntPredicate;

use crate::codegen::CodeGen;
use types::instruction::Instruction;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_binops(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let binop_fn_type = void_type.fn_type(&[], false);
        let binop_fn = match instr {
            Instruction::Add
            | Instruction::Sub
            | Instruction::Div
            | Instruction::Mul
            | Instruction::Mod
            | Instruction::Gt => {
                self.module
                    .add_function(instr.to_llvm_name(), binop_fn_type, None)
            }
            _ => panic!("Not a binary operation!"),
        };

        // i64s are 64 bits, so we want to do stack[stack_size - 1] + stack[stack_size - 2] if possible
        // Basic blocks (Only some will be used depending on the operation)
        let basic_block = self.context.append_basic_block(binop_fn, "");
        self.builder.position_at_end(basic_block);
        let cont_block = self.context.append_basic_block(binop_fn, "cont");
        let dividend_nonzero = self
            .context
            .append_basic_block(binop_fn, "dividend_nonzero");
        let then_block = self.context.append_basic_block(binop_fn, "");
        let else_block = self.context.append_basic_block(binop_fn, "");
        let ret_block = self.context.insert_basic_block_after(else_block, "ret");

        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let load_piet_stack = self
            .builder
            .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
            .unwrap()
            .into_pointer_value();

        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_2 = self.context.i64_type().const_int(2, false);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let stack_size_val = self
            .builder
            .build_load(self.context.i64_type(),stack_size_addr, "stack_size")
            .unwrap()
            .into_int_value();

        let cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_2,
            "check_stack_size",
        ).unwrap();

        let _ = self.builder
            .build_conditional_branch(cmp, cont_block, ret_block);

        // Enough elems on stack
        self.builder.position_at_end(cont_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx");

        let top_ptr_gep = unsafe { self.builder.build_gep(self.context.i64_type(), load_piet_stack, &[top_idx.unwrap()], "").unwrap() };

        let next_idx = self
            .builder
            .build_int_sub(stack_size_val, const_2, "next_elem_idx")
            .unwrap();

        let next_ptr = unsafe { self.builder.build_gep(self.context.i64_type(), load_piet_stack, &[next_idx], "").unwrap() };

        let top_ptr_val = self
            .builder
            .build_load(self.context.i64_type(), top_ptr_gep, "top_elem_val")
            .unwrap()
            .into_int_value();

        let next_ptr_val = self
            .builder
            .build_load(self.context.i64_type(), next_ptr, "next_elem_val")
            .unwrap()
            .into_int_value();

        let result = match instr {
            Instruction::Add => {
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };
                self.builder.build_int_add(next_ptr_val, top_ptr_val, "add").unwrap()
            }
            Instruction::Sub => {
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };
                self.builder.build_int_sub(next_ptr_val, top_ptr_val, "sub").unwrap()
            }
            Instruction::Mul => {
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };
                self.builder.build_int_mul(next_ptr_val, top_ptr_val, "mul").unwrap()
            }
            Instruction::Div => {
                let cmp = self.builder.build_int_compare(
                    IntPredicate::NE,
                    top_ptr_val,
                    const_0,
                    "check_dividend_nonzero",
                );
                let _ = self.builder
                    .build_conditional_branch(cmp.unwrap(), dividend_nonzero, ret_block);

                // Set names for blocks and delete unused
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };

                self.builder.position_at_end(dividend_nonzero);
                self.builder
                    .build_int_signed_div(next_ptr_val, top_ptr_val, "div").unwrap()
            }
            Instruction::Mod => {
                let i64_type = self.context.i64_type();

                // Calculate the absolute value without a branch
                let shift_amount = i64_type.const_int(63, false);
                let a_asr = self.builder.build_right_shift(
                    top_ptr_val,
                    shift_amount,
                    true,
                    "arithmetic_right_shift",
                ).unwrap();
                let a_xor = self.builder.build_xor(top_ptr_val, a_asr, "a_xor");
                let top_ptr_val = self.builder.build_int_sub(a_xor.unwrap(), a_asr, "abs_a").unwrap();

                let cmp = self.builder.build_int_compare(
                    IntPredicate::NE,
                    top_ptr_val,
                    const_0,
                    "check_dividend_nonzero",
                );

                let _ = self.builder
                    .build_conditional_branch(cmp.unwrap(), dividend_nonzero, ret_block);

                self.builder.position_at_end(dividend_nonzero);
                let rem = self
                    .builder
                    .build_int_signed_rem(next_ptr_val, top_ptr_val, "mod")
                    .unwrap();

                let store_rem_result = self
                    .builder
                    .build_alloca(self.context.i64_type(), "rem_result")
                    .unwrap();
                let _ = self.builder.build_store(store_rem_result, rem);

                let cmp = self.builder.build_int_compare(
                    IntPredicate::SGE,
                    rem,
                    const_0,
                    "check_mod_sign",
                );

                then_block.set_name("lz");
                else_block.set_name("gez");

                let _ = self.builder
                    .build_conditional_branch(cmp.unwrap(), else_block, then_block);

                self.builder.position_at_end(then_block);
                let rem_lz = self.builder.build_int_add(top_ptr_val, rem, "rem_lz");
                let _ = self.builder.build_store(store_rem_result, rem_lz.unwrap());
                let _ = self.builder.build_unconditional_branch(else_block);
                self.builder.position_at_end(else_block);
                self.builder
                    .build_load(self.context.i64_type(), store_rem_result, "load_result")
                    .unwrap()
                    .into_int_value()
            }
            Instruction::Gt => {
                // Pushes 1 to stack if second top > top otherwise 0
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };

                let diff = self.builder.build_int_sub(next_ptr_val, top_ptr_val, "sub");
                let value_cmp = self.builder.build_int_compare(
                    IntPredicate::SGT,
                    diff.unwrap(),
                    const_0,
                    "check_next_gt_top",
                );
                let res = self.builder.build_int_z_extend(
                    value_cmp.unwrap(),
                    self.context.i64_type(),
                    "zero_extend_cmp",
                );

                res.unwrap()
            }
            _ => panic!("Not a binary operation!"),
        };

        let updated_stack_size =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size");

        let _ = self.builder
            .build_store(stack_size_addr, updated_stack_size.unwrap());

        let _ = self.builder.build_store(next_ptr, result);

        let _ = self.builder.build_unconditional_branch(ret_block);
        // Not enough elems on stack
        self.builder.position_at_end(ret_block);
        let _ = self.builder.build_return(None);
    }
}
