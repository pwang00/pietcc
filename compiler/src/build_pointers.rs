use crate::codegen::CodeGen;
use inkwell::IntPredicate;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_switch(&self) {
        let void_type = self.context.void_type();
        let switch_fn_type = void_type.fn_type(&[], false);
        let switch_fn = self
            .module
            .add_function("piet_switch", switch_fn_type, None);

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_2 = self.context.i64_type().const_int(2, false);

        let basic_block = self.context.append_basic_block(switch_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(switch_fn, "stack_nonempty");
        let lz_block = self.context.append_basic_block(switch_fn, "mod_lz");
        let gz_block = self.context.append_basic_block(switch_fn, "mod_gz");
        let ret_block = self.context.insert_basic_block_after(gz_block, "ret");

        let cc_addr = self.module.get_global("cc").unwrap().as_pointer_value();

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
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let stack_size_cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        );

        self.builder
            .build_conditional_branch(stack_size_cmp, then_block, ret_block);
        self.builder.position_at_end(then_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx");

        let load_piet_stack = self
            .builder
            .build_load(stack_addr, "load_piet_stack")
            .into_pointer_value();
        let top_ptr_gep = unsafe { self.builder.build_gep(load_piet_stack, &[top_idx], "") };
        let top_ptr_val = self
            .builder
            .build_load(top_ptr_gep, "top_elem_val")
            .into_int_value();

        let res = self
            .builder
            .build_int_signed_rem(top_ptr_val, const_2, "mod_by_2");

        // If rem < 0 then we store -rem, otherwise we store rem
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SGE, res, const_0, "cmp_mod_gz");
        self.builder
            .build_conditional_branch(cmp, lz_block, gz_block);
        self.builder.position_at_end(lz_block);
        let rem_lz = self.builder.build_int_neg(res, "neg_res");
        let rem_lz = self
            .builder
            .build_int_truncate(rem_lz, self.context.i8_type(), "trunc_to_i8");
        self.builder.build_store(cc_addr, rem_lz);
        self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(gz_block);
        let rem_gz = self
            .builder
            .build_int_truncate(res, self.context.i8_type(), "trunc_to_i8");
        self.builder.build_store(cc_addr, rem_gz);
        self.builder.build_unconditional_branch(ret_block);
        // Return
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub fn build_rotate(&self) {
        let void_type = self.context.void_type();
        let rotate_fn_type = void_type.fn_type(&[], false);
        let rotate_fn = self
            .module
            .add_function("piet_rotate", rotate_fn_type, None);

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_4 = self.context.i64_type().const_int(4, false);

        let basic_block = self.context.append_basic_block(rotate_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(rotate_fn, "stack_nonempty");

        let lz_block = self.context.append_basic_block(rotate_fn, "top_lz");
        let gz_block = self.context.append_basic_block(rotate_fn, "top_gz");
        let ret_block = self.context.insert_basic_block_after(gz_block, "ret");

        let dp_addr = self.module.get_global("dp").unwrap().as_pointer_value();

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
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let stack_size_cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        );

        self.builder
            .build_conditional_branch(stack_size_cmp, then_block, ret_block);
        self.builder.position_at_end(then_block);

        let top_idx = self
            .builder
            .build_int_sub(stack_size_val, const_1, "top_elem_idx");

        let load_piet_stack = self
            .builder
            .build_load(stack_addr, "load_piet_stack")
            .into_pointer_value();
        let top_ptr_gep = unsafe { self.builder.build_gep(load_piet_stack, &[top_idx], "") };

        let top_ptr_val = self
            .builder
            .build_load(top_ptr_gep, "top_elem_val")
            .into_int_value();

        let rem = self
            .builder
            .build_int_signed_rem(top_ptr_val, const_4, "rem");

        let cmp =
            self.builder
                .build_int_compare(IntPredicate::SGE, top_ptr_val, const_0, "cmp_top_zero");

        self.builder
            .build_conditional_branch(cmp, lz_block, gz_block);
        self.builder.position_at_end(lz_block);

        let rem_lz = self.builder.build_int_add(rem, const_4, "updated_rem");
        let rem_lz = self
            .builder
            .build_int_truncate(rem_lz, self.context.i8_type(), "trunc_to_i8");
        self.builder.build_store(dp_addr, rem_lz);
        self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(gz_block);
        let rem_gz = self
            .builder
            .build_int_truncate(rem, self.context.i8_type(), "trunc_to_i8");
        self.builder.build_store(dp_addr, rem_gz);
        self.builder.build_unconditional_branch(ret_block);
        // Return
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}
