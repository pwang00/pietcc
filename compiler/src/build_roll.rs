use inkwell::{
    values::{AnyValue, BasicValue, IntValue, PointerValue},
    IntPredicate,
};

use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_roll(&self) {
        // Types:
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();

        // Roll function type
        let roll_fn_type = void_type.fn_type(&[], false);
        let roll_fn = self.module.add_function("piet_roll", roll_fn_type, None);

        // LLVM intrinsics
        let stack_save_fn = self.module.get_function("llvm.stacksave").unwrap();
        let stack_restore_fn = self.module.get_function("llvm.stackrestore").unwrap();
        let llvm_smax_fn = self.module.get_function("llvm.smax.i64").unwrap();

        // Globals
        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();
        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        // Constants
        let const_neg_1 = self.context.i64_type().const_int(u64::MAX, true);
        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_2 = self.context.i64_type().const_int(2, false);

        // Basic blocks
        let entry = self.context.append_basic_block(roll_fn, "");
        self.builder.position_at_end(entry);
        let block_3 = self.context.append_basic_block(roll_fn, "");
        let block_14 = self.context.append_basic_block(roll_fn, "");
        let block_18 = self.context.append_basic_block(roll_fn, "");
        let block_22 = self.context.append_basic_block(roll_fn, "");
        let block_27 = self.context.append_basic_block(roll_fn, "");
        let block_30 = self.context.append_basic_block(roll_fn, "");
        let block_32 = self.context.append_basic_block(roll_fn, "");
        let block_41 = self.context.append_basic_block(roll_fn, "");
        let block_44 = self.context.append_basic_block(roll_fn, "");
        let block_45 = self.context.append_basic_block(roll_fn, "");
        let block_55 = self.context.append_basic_block(roll_fn, "");

        // Load stack size
        let stack_size_load_instr = self
            .builder
            .build_load(stack_size_addr, "")
            .as_instruction_value()
            .unwrap();
        stack_size_load_instr.set_alignment(8).ok();
        let stack_size_val = stack_size_load_instr.try_into().unwrap();

        let cmp = self
            .builder
            .build_int_compare(IntPredicate::SLT, stack_size_val, const_2, "");
        self.builder
            .build_conditional_branch(cmp, block_55, block_3);

        // Block 3
        self.builder.position_at_end(block_3);
        let load_piet_stack = self
            .builder
            .build_load(stack_addr, "")
            .as_instruction_value()
            .unwrap();
        load_piet_stack.set_alignment(8).ok();
        let load_piet_stack = load_piet_stack.try_into().unwrap();

        let top_idx = self.builder.build_int_nsw_sub(stack_size_val, const_1, "");
        let top_elem_addr = unsafe {
            self.builder
                .build_in_bounds_gep(load_piet_stack, &[top_idx], "")
        };
        let top_elem_load_instr = self
            .builder
            .build_load(top_elem_addr, "")
            .as_instruction_value()
            .unwrap();
        top_elem_load_instr.set_alignment(8).ok();
        let top_elem_val = top_elem_load_instr.try_into().unwrap();

        let size_minus_two = self.builder.build_int_nsw_sub(stack_size_val, const_2, "");
        let next_elem_addr = unsafe {
            self.builder
                .build_in_bounds_gep(load_piet_stack, &[size_minus_two], "")
        };
        let next_elem_load_instr = self
            .builder
            .build_load(next_elem_addr, "")
            .as_instruction_value()
            .unwrap();
        next_elem_load_instr.set_alignment(8).ok();
        let next_elem_val = next_elem_load_instr.try_into().unwrap();

        let store = self.builder.build_store(stack_size_addr, size_minus_two);
        store.set_alignment(8).ok();

        let left =
            self.builder
                .build_int_compare(IntPredicate::SLT, size_minus_two, next_elem_val, "");
        let right = self
            .builder
            .build_int_compare(IntPredicate::SLT, next_elem_val, const_0, "");
        let or = self.builder.build_or(left, right, "");
        self.builder
            .build_conditional_branch(or, block_55, block_14);

        // Block 14
        self.builder.position_at_end(block_14);
        let stack_save = self.builder.build_call(stack_save_fn, &[], "");
        stack_save.set_tail_call(true);
        let stack_alloc = self
            .builder
            .build_array_alloca(i64_type, next_elem_val, "")
            .as_instruction_value()
            .unwrap();
        stack_alloc.set_alignment(16).ok();
        let rolls_ltz =
            self.builder
                .build_int_compare(IntPredicate::SLT, top_elem_val, const_0, "");
        self.builder
            .build_conditional_branch(rolls_ltz, block_18, block_22);

        // Block 18
        self.builder.position_at_end(block_18);
        let sub1 = self.builder.build_int_sub(const_0, top_elem_val, "");
        let rem = self.builder.build_int_unsigned_rem(sub1, next_elem_val, "");
        let sub2 = self.builder.build_int_nsw_sub(next_elem_val, rem, "");
        self.builder.build_unconditional_branch(block_22);

        // Block 22
        self.builder.position_at_end(block_22);
        let phi1 = self.builder.build_phi(i64_type, "");
        phi1.add_incoming(&[
            (&sub2.as_basic_value_enum(), block_18),
            (&top_elem_val.as_basic_value_enum(), block_14),
        ]);

        let add = self.builder.build_int_nsw_add(
            phi1.as_basic_value().into_int_value(),
            next_elem_val,
            "",
        );

        let load_piet_stack2 = self
            .builder
            .build_load(stack_addr, "")
            .as_instruction_value()
            .unwrap();
        load_piet_stack2.set_alignment(8).ok();
        let load_piet_stack2: PointerValue = load_piet_stack2.try_into().unwrap();

        let stack_size_load_instr2 = self
            .builder
            .build_load(stack_size_addr, "")
            .as_instruction_value()
            .unwrap();
        stack_size_load_instr2.set_alignment(8).ok();
        let stack_size_val2: IntValue = stack_size_load_instr2.try_into().unwrap();
        self.builder.build_unconditional_branch(block_27);

        // Block 27
        self.builder.position_at_end(block_27);
        let phi2 = self.builder.build_phi(i64_type, "");
        phi2.add_incoming(&[(&phi1.as_basic_value(), block_22)]);
        let cmp3 = self.builder.build_int_compare(
            IntPredicate::SLT,
            phi2.as_basic_value().into_int_value(),
            add,
            "",
        );
        self.builder
            .build_conditional_branch(cmp3, block_32, block_30);

        // Block 30
        self.builder.position_at_end(block_30);
        let call2 =
            self.builder
                .build_call(llvm_smax_fn, &[next_elem_val.into(), const_0.into()], "");
        self.builder.build_unconditional_branch(block_41);

        // Block 32
        self.builder.position_at_end(block_32);
        let rem2 = self.builder.build_int_signed_rem(
            phi2.as_basic_value().into_int_value(),
            next_elem_val,
            "",
        );
        let sign_change = self.builder.build_xor(rem2, const_neg_1, "");
        let make_positive = self.builder.build_int_add(stack_size_val2, sign_change, "");
        let curr_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(load_piet_stack2, &[make_positive], "")
        };
        let curr_ptr_load_instr = self
            .builder
            .build_load(curr_ptr, "")
            .as_instruction_value()
            .unwrap();
        curr_ptr_load_instr.set_alignment(8).ok();
        let curr_val: IntValue = curr_ptr_load_instr.try_into().unwrap();
        let sub_phis = self.builder.build_int_nsw_sub(
            phi2.as_basic_value().into_int_value(),
            phi1.as_basic_value().into_int_value(),
            "",
        );
        let slice_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(stack_alloc.try_into().unwrap(), &[sub_phis], "")
        };
        let store_in_slice = self.builder.build_store(slice_ptr, curr_val);
        store_in_slice.set_alignment(8).ok();
        let incr_idx =
            self.builder
                .build_int_nsw_add(phi2.as_basic_value().into_int_value(), const_1, "");
        phi2.add_incoming(&[(&incr_idx.as_basic_value_enum(), block_32)]);
        self.builder.build_unconditional_branch(block_27);

        // Block 41
        self.builder.position_at_end(block_41);
        let phi3 = self.builder.build_phi(i64_type, "");
        let cmp3 = self.builder.build_int_compare(
            IntPredicate::EQ,
            phi3.as_basic_value().into_int_value(),
            call2.as_any_value_enum().into_int_value(),
            "",
        );
        self.builder
            .build_conditional_branch(cmp3, block_44, block_45);

        // Block 44
        self.builder.position_at_end(block_44);
        let restore = self.builder.build_call(
            stack_restore_fn,
            &[stack_save.try_as_basic_value().unwrap_left().into()],
            "",
        );
        restore.set_tail_call(true);
        self.builder.build_unconditional_branch(block_55);

        // Block 45
        self.builder.position_at_end(block_45);
        let sign_change =
            self.builder
                .build_xor(phi3.as_basic_value().into_int_value(), const_neg_1, "");
        let add = self.builder.build_int_add(next_elem_val, sign_change, "");
        let slice_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(stack_alloc.try_into().unwrap(), &[add], "")
        };
        let slice_elem_load_instr = self
            .builder
            .build_load(slice_ptr, "")
            .as_instruction_value()
            .unwrap();
        slice_elem_load_instr.set_alignment(8).ok();
        let slice_elem: IntValue = slice_elem_load_instr.try_into().unwrap();

        let stack_size_load_instr3 = self
            .builder
            .build_load(stack_size_addr, "")
            .as_instruction_value()
            .unwrap();
        stack_size_load_instr3.set_alignment(8).ok();
        let stack_size_val: IntValue = stack_size_load_instr3.try_into().unwrap();

        let sub_idx =
            self.builder
                .build_int_sub(phi3.as_basic_value().into_int_value(), next_elem_val, "");
        let add_idx = self.builder.build_int_add(sub_idx, stack_size_val, "");

        let curr_stack_ptr = unsafe {
            self.builder
                .build_in_bounds_gep(load_piet_stack2, &[add_idx], "")
        };
        let store_in_stack = self.builder.build_store(curr_stack_ptr, slice_elem);
        store_in_stack.set_alignment(8).ok();
        let incr_idx =
            self.builder
                .build_int_nuw_add(phi3.as_basic_value().into_int_value(), const_1, "");
        phi3.add_incoming(&[
            (&incr_idx.as_basic_value_enum(), block_45),
            (&const_0, block_30),
        ]);
        self.builder.build_unconditional_branch(block_41);
        // Return
        self.builder.position_at_end(block_55);
        self.builder.build_return(None);
    }
}
