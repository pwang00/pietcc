use crate::codegen::CodeGen;
use inkwell::IntPredicate;
use piet_core::instruction::Instruction;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_switch(&self) {
        let switch_fn =
            self.module
                .add_function(Instruction::Swi.to_llvm_name(), self.context.void_type().fn_type(&[], false), None);

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_2 = self.context.i64_type().const_int(2, false);
        let const_2_i8 = self.context.i8_type().const_int(2, false);
        let basic_block = self.context.append_basic_block(switch_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(switch_fn, "stack_nonempty");
        let lz_block = self.context.append_basic_block(switch_fn, "mod_lz");
        let gz_block = self.context.append_basic_block(switch_fn, "mod_gz");
        let ret_block = self.context.insert_basic_block_after(gz_block, "ret");

        let cc_addr = self.module.get_global("cc").unwrap().as_pointer_value();
        let cc_val = self.builder.build_load(self.context.i64_type(), cc_addr, "")
            .unwrap()
            .into_int_value();

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

        let stack_size_val = self
            .builder
            .build_load(self.context.i64_type(), stack_size_addr, "stack_size")
            .unwrap()
            .into_int_value();

        let stack_size_cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        ).unwrap();

        let _ = self.builder
            .build_conditional_branch(stack_size_cmp, then_block, ret_block);
        self.builder.position_at_end(then_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx")
            .unwrap();

        let load_piet_stack = self
            .builder
            .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack")
            .unwrap()
            .into_pointer_value();
        let top_ptr_gep = unsafe { self.builder.build_gep(self.context.i64_type(), load_piet_stack, &[top_idx], "").unwrap() };
        let top_ptr_val = self
            .builder
            .build_load(self.context.i64_type(), top_ptr_gep, "top_elem_val")
            .unwrap()
            .into_int_value();

        let res = self
            .builder
            .build_int_signed_rem(top_ptr_val, const_2, "mod_by_2")
            .unwrap();

        // If rem < 0 then we store -rem, otherwise we store rem
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SGE, res, const_0, "cmp_mod_gz")
            .unwrap();
        let _ = self.builder
            .build_conditional_branch(cmp, lz_block, gz_block);
        self.builder.position_at_end(lz_block);
        let decremented =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
                .unwrap();
            let _ = self.builder.build_store(stack_size_addr, decremented);
        let rem_lz = self.builder.build_int_neg(res, "neg_res").unwrap();
        let rem_lz = self
            .builder
            .build_int_truncate(rem_lz, self.context.i8_type(), "trunc_to_i8").unwrap();
        let rem_lz = self.builder.build_int_add(rem_lz, cc_val, "").unwrap();
        let rem_lz = self.builder.build_int_unsigned_rem(rem_lz, const_2_i8, "").unwrap();
        let _ = self.builder.build_store(cc_addr, rem_lz);
        let _ = self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(gz_block);
        let decremented =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size").unwrap();
        self.builder.build_store(stack_size_addr, decremented)
            .unwrap();
        let rem_gz = self
            .builder
            .build_int_truncate(res, self.context.i8_type(), "trunc_to_i8")
            .unwrap();
        let rem_gz = self.builder.build_int_add(rem_gz, cc_val, "").unwrap();
        let rem_gz = self.builder.build_int_unsigned_rem(rem_gz, const_2_i8, "").unwrap();
        let _ = self.builder.build_store(cc_addr, rem_gz);
        let _ = self.builder.build_unconditional_branch(ret_block);
        // Return
        self.builder.position_at_end(ret_block);
        let _ = self.builder.build_return(None);
    }

    pub(crate) fn build_rotate(&self) {
        let rotate_fn_type = self.context.void_type().fn_type(&[], false);
        let rotate_fn = self
            .module
            .add_function("piet_rotate", rotate_fn_type, None);

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_4 = self.context.i64_type().const_int(4, false);
        let const_4_i8 = self.context.i8_type().const_int(4, false);
        let basic_block = self.context.append_basic_block(rotate_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(rotate_fn, "stack_nonempty");

        let lz_block = self.context.append_basic_block(rotate_fn, "top_lz");
        let gz_block = self.context.append_basic_block(rotate_fn, "top_gz");
        let ret_block = self.context.insert_basic_block_after(gz_block, "ret");

        let dp_addr = self.module.get_global("dp").unwrap().as_pointer_value();
        let dp_val = self.builder.build_load(self.context.i64_type(), dp_addr, "").unwrap().into_int_value();
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

        let stack_size_val = self
            .builder
            .build_load(self.context.i64_type(), stack_size_addr, "stack_size")
            .unwrap()
            .into_int_value();

        let stack_size_cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        ).unwrap();

        let _ = self.builder
            .build_conditional_branch(stack_size_cmp, then_block, ret_block);
        self.builder.position_at_end(then_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx")
            .unwrap();

        let load_piet_stack = self
            .builder
            .build_load(stack_addr.get_type(), stack_addr, "load_piet_stack").unwrap()
            .into_pointer_value();
        let top_ptr_gep = unsafe { self.builder.build_gep(self.context.i64_type(), load_piet_stack, &[top_idx], "").unwrap() };

        let top_ptr_val = self
            .builder
            .build_load(self.context.i64_type(), top_ptr_gep, "top_elem_val")
            .unwrap()
            .into_int_value();

        let rem = self
            .builder
            .build_int_signed_rem(top_ptr_val, const_4, "rem")
            .unwrap();

        let cmp =
            self.builder
                .build_int_compare(IntPredicate::SGE, top_ptr_val, const_0, "cmp_top_zero")
                .unwrap();

        let _ = self.builder
            .build_conditional_branch(cmp, gz_block, lz_block);
        self.builder.position_at_end(lz_block);
        let decremented =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
                .unwrap();
        self.builder.build_store(stack_size_addr, decremented).unwrap();
        let rem_lz = self.builder.build_int_add(rem, const_4, "updated_rem").unwrap();
        let rem_lz = self
            .builder
            .build_int_truncate(rem_lz, self.context.i8_type(), "trunc_to_i8").unwrap();
        let rem_lz = self.builder.build_int_add(rem_lz, dp_val, "").unwrap();
        let rem_lz = self.builder.build_int_unsigned_rem(rem_lz, const_4_i8, "").unwrap();
        let _ = self.builder.build_store(dp_addr, rem_lz);
        let _ = self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(gz_block);
        let decremented =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size")
                .unwrap();
        let _ = self.builder.build_store(stack_size_addr, decremented);
        let rem_gz = self
            .builder
            .build_int_truncate(rem, self.context.i8_type(), "trunc_to_i8")
            .unwrap();
        let rem_gz = self.builder.build_int_add(rem_gz, dp_val, "").unwrap();
        let rem_gz = self.builder.build_int_unsigned_rem(rem_gz, const_4_i8, "").unwrap();
        let _ = self.builder.build_store(dp_addr, rem_gz);
        let _ = self.builder.build_unconditional_branch(ret_block);
        // Return
        self.builder.position_at_end(ret_block);
        let _ = self.builder.build_return(None);
    }

    pub(crate) fn build_retry(&self) {
        let void_type = self.context.void_type();
        let i8_type = self.context.i8_type();
        let retry_fn_type = void_type.fn_type(&[], false);
        let retry_fn = self.module.add_function("retry", retry_fn_type, None);
        // Basic blocks
        let basic_block = self.context.append_basic_block(retry_fn, "");
        let one_mod_two = self.context.append_basic_block(retry_fn, "one_mod_two");
        let zero_mod_two = self.context.append_basic_block(retry_fn, "zero_mod_two");
        let ret_block = self.context.append_basic_block(retry_fn, "ret");

        self.builder.position_at_end(basic_block);
        // Constants
        let const_1 = i8_type.const_int(1, false);
        let const_2 = i8_type.const_int(2, false);
        let const_4 = i8_type.const_int(4, false);
        let const_8 = i8_type.const_int(8, false);
        // Pointers dp and cc
        let dp_addr = self.module.get_global("dp").unwrap().as_pointer_value();
        let cc_addr = self.module.get_global("cc").unwrap().as_pointer_value();
        let rctr_addr = self.module.get_global("rctr").unwrap().as_pointer_value();

        // Loaded values
        let dp_val = self.builder.build_load(self.context.i64_type(), dp_addr, "load_dp").unwrap().into_int_value();
        let cc_val = self.builder.build_load(self.context.i64_type(), cc_addr, "load_cc").unwrap().into_int_value();
        let rctr_val = self
            .builder
            .build_load(self.context.i64_type(), rctr_addr, "load_rctr")
            .unwrap()
            .into_int_value();

        let rem = self.builder.build_int_unsigned_rem(rctr_val, const_2, "")
            .unwrap();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::EQ, rem, const_1, "")
            .unwrap();
        let _ = self.builder
            .build_conditional_branch(cmp, one_mod_two, zero_mod_two);

        // One mod two
        self.builder.position_at_end(one_mod_two);
        let dp_sum = self.builder.build_int_add(dp_val, const_1, "rotate_dp").unwrap();
        let dp_mod_4 = self
            .builder
            .build_int_unsigned_rem(dp_sum, const_4, "dp_mod_4")
            .unwrap();
        let dp_to_i8 =
            self.builder
                .build_int_truncate(dp_mod_4, self.context.i8_type(), "trunc_dp_to_i8")
                .unwrap();
        let _ = self.builder.build_store(dp_addr, dp_to_i8);
        let _ = self.builder.build_unconditional_branch(ret_block);

        // Zero mod two
        self.builder.position_at_end(zero_mod_two);
        let cc_sum = self.builder.build_int_add(cc_val, const_1, "rotate_cc").unwrap();
        let cc_mod_2 = self
            .builder
            .build_int_unsigned_rem(cc_sum, const_2, "dp_mod_4")
            .unwrap();
        let cc_to_i8 =
            self.builder
                .build_int_truncate(cc_mod_2, self.context.i8_type(), "trunc_cc_to_i8")
                .unwrap();
        let _ = self.builder.build_store(cc_addr, cc_to_i8);
        let _ = self.builder.build_unconditional_branch(ret_block);

        // Ret
        self.builder.position_at_end(ret_block);
        //self.builder.build_call(print_ptr_fn, &[], "");
        let rctr_added = self.builder.build_int_add(rctr_val, const_1, "").unwrap();
        let rctr_mod_8 = self.builder.build_int_unsigned_rem(rctr_added, const_8, "").unwrap();
        let rctr_to_i8 =
            self.builder
                .build_int_truncate(rctr_mod_8, self.context.i8_type(), "trunc_to_i8").unwrap();
        let _ = self.builder.build_store(rctr_addr, rctr_to_i8);
        let _ = self.builder.build_return(None);
    }
}
