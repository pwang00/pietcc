use inkwell::IntPredicate;
use crate::codegen::CodeGen;

#[allow(unused)]
impl<'a, 'b> CodeGen<'a, 'b> {
    pub fn build_switch(&self) {
        let void_type = self.context.void_type();
        let rotate_fn_type = void_type.fn_type(&[], false);
        let rotate_fn = self
            .module
            .add_function("piet_rotate", rotate_fn_type, None);

        let const_1 = self.context.i64_type().const_int(1, false);
        let const_2 = self.context.i64_type().const_int(2, false);

        let basic_block = self.context.append_basic_block(rotate_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(rotate_fn, "");
        let else_block = self.context.append_basic_block(rotate_fn, "");
        let ret_block = self.context.insert_basic_block_after(else_block, "ret");

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

        let top_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };

        let top_ptr_deref1 = self.builder.build_load(top_ptr_gep, "top_elem_deref1");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr_deref1.into_pointer_value(), "top_elem_val")
            .into_int_value();

        let res = self
            .builder
            .build_int_unsigned_rem(top_ptr_val, const_2, "mod_by_2");
        self.builder.build_store(cc_addr, res);

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

        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_4 = self.context.i64_type().const_int(4, false);

        let basic_block = self.context.append_basic_block(rotate_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(rotate_fn, "");
        let else_block = self.context.append_basic_block(rotate_fn, "");
        let ret_block = self.context.insert_basic_block_after(else_block, "ret");

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

        let top_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };

        let top_ptr_deref1 = self.builder.build_load(top_ptr_gep, "top_elem_deref1");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr_deref1.into_pointer_value(), "top_elem_val");

        // Rotate by modulus + x if x is negative, otherwise just x
        let rem = self
            .builder
            .build_int_signed_rem(top_ptr_val.into_int_value(), const_4, "mod")
            .const_z_ext(self.context.i64_type());

        /* Modulo is just
           if remainder > 0 then remainder
           else modulus + remainder
        */
        let store_rem_result = self
            .builder
            .build_alloca(self.context.i64_type(), "rem_result");

        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SGE, rem, const_0, "check_mod_sign");

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
        let res = self
            .builder
            .build_load(store_rem_result, "load_result")
            .into_int_value();

        let updated_dp = self
            .builder
            .build_int_unsigned_rem(res, const_4, "mod_by_4");
        self.builder.build_store(dp_addr, updated_dp);

        // Return
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}