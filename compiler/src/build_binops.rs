use inkwell::{
    IntPredicate,
};

use crate::codegen::CodeGen;
use types::instruction::Instruction;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_binops(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let binop_fn_type = void_type.fn_type(&[], false);
        let binop_fn = match instr {
            Instruction::Add => self.module.add_function("piet_add", binop_fn_type, None),
            Instruction::Sub => self.module.add_function("piet_sub", binop_fn_type, None),
            Instruction::Div => self.module.add_function("piet_div", binop_fn_type, None),
            Instruction::Mul => self.module.add_function("piet_mul", binop_fn_type, None),
            Instruction::Mod => self.module.add_function("piet_mod", binop_fn_type, None),
            Instruction::Gt => self.module.add_function("piet_gt", binop_fn_type, None),
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

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_2 = self.context.i64_type().const_int(2, false);

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_2,
            "check_stack_size",
        );

        self.builder
            .build_conditional_branch(cmp, cont_block, ret_block);

        // Enough elems on stack
        self.builder.position_at_end(cont_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx");

        let top_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };

        let next_idx = self
            .builder
            .build_int_sub(stack_size_val, const_2, "next_elem_idx");

        let next_ptr = unsafe { self.builder.build_gep(stack_addr, &[next_idx], "") };

        let top_ptr = self.builder.build_load(top_ptr_gep, "top_elem_ptr");
        let next_ptr = self.builder.build_load(next_ptr, "next_elem_ptr");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr.into_pointer_value(), "top_elem_val");

        let next_ptr_val = self
            .builder
            .build_load(next_ptr.into_pointer_value(), "next_elem_val");

        let result = match instr {
            Instruction::Add => {
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };
                self.builder.build_int_add(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "add",
                )
            }
            Instruction::Sub => {
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };
                self.builder.build_int_sub(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "sub",
                )
            }
            Instruction::Mul => {
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };
                self.builder.build_int_mul(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "mul",
                )
            }
            Instruction::Div => {
                let cmp = self.builder.build_int_compare(
                    IntPredicate::NE,
                    top_ptr_val.into_int_value(),
                    const_0,
                    "check_dividend_nonzero",
                );
                self.builder
                    .build_conditional_branch(cmp, then_block, ret_block);

                // Set names for blocks and delete unused
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };

                self.builder.position_at_end(dividend_nonzero);
                self.builder.build_int_signed_div(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "div",
                )
            }
            Instruction::Mod => {
                let cmp = self.builder.build_int_compare(
                    IntPredicate::NE,
                    top_ptr_val.into_int_value(),
                    const_0,
                    "check_dividend_nonzero",
                );

                self.builder
                    .build_conditional_branch(cmp, then_block, ret_block);

                self.builder.position_at_end(dividend_nonzero);
                let rem = self.builder.build_int_signed_rem(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "mod",
                );

                /* Modulo is just
                   if remainder > 0 then remainder
                   else modulus + remainder
                */

                let store_rem_result = self
                    .builder
                    .build_alloca(self.context.i64_type(), "rem_result");

                let cmp = self.builder.build_int_compare(
                    IntPredicate::SGE,
                    rem,
                    const_0,
                    "check_mod_sign",
                );

                then_block.set_name("lz");
                else_block.set_name("gez");

                self.builder
                    .build_conditional_branch(cmp, else_block, then_block);

                self.builder.position_at_end(then_block);
                let rem = self.builder.build_int_add(const_0, rem, "rem_lz");
                self.builder.build_store(store_rem_result, rem);
                self.builder.position_at_end(else_block);
                let rem = self
                    .builder
                    .build_int_add(top_ptr_val.into_int_value(), rem, "rem_gz");
                self.builder.build_store(store_rem_result, rem);
                self.builder
                    .build_load(store_rem_result, "load_result")
                    .into_int_value()
            }
            Instruction::Gt => {
                // Pushes 1 to stack if second top > top otherwise 0
                unsafe { then_block.delete().ok() };
                unsafe { else_block.delete().ok() };
                unsafe { dividend_nonzero.delete().ok() };

                let diff = self.builder.build_int_sub(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "sub",
                );
                let value_cmp = self.builder.build_int_compare(
                    IntPredicate::SGT,
                    diff,
                    const_0,
                    "check_next_gt_top",
                );
                let res = self.builder.build_int_z_extend(
                    value_cmp,
                    self.context.i64_type(),
                    "zero_extend_cmp",
                );

                res
            }
            _ => panic!("Not a binary operation!"),
        };

        let updated_stack_size =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size");

        self.builder
            .build_store(stack_size_addr, updated_stack_size);

        self.builder
            .build_store(next_ptr.into_pointer_value(), result);

        self.builder.build_unconditional_branch(ret_block);
        // Not enough elems on stack
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}