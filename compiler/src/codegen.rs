use crate::cfg::CFGGenerator;
use crate::consts::STACK_SIZE;
use inkwell::{
    basic_block,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::BasicMetadataTypeEnum::IntType,
    types::BasicMetadataTypeEnum::PointerType,
    types::BasicType,
    values::{ArrayValue, BasicValue, FunctionValue, IntValue},
    AddressSpace, IntPredicate,
};
use types::instruction::Instruction;

pub struct CodeGen<'a, 'b> {
    context: &'b Context,
    module: Module<'b>,
    builder: Builder<'b>,
    cfg: CFGGenerator<'a>,
}

impl<'a, 'b> CodeGen<'a, 'b> {
    pub fn new(
        context: &'b Context,
        module: Module<'b>,
        builder: Builder<'b>,
        cfg: CFGGenerator<'a>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            cfg,
        }
    }

    pub fn build_push(&self) {
        let void_type = self.context.void_type();
        let push_fn_type = void_type.fn_type(&[IntType(self.context.i64_type())], false);
        let push_fn = self.module.add_function("piet_push", push_fn_type, None);
        let basic_block = self.context.append_basic_block(push_fn, "");
        self.builder.position_at_end(basic_block);

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

        let const_1 = self.context.i64_type().const_int(1, false);

        let top_ptr = self.builder.build_load(
            unsafe { self.builder.build_gep(stack_addr, &[stack_size_val], "") },
            "top_elem_ptr",
        );

        let first_param = push_fn.get_first_param().unwrap();
        self.builder
            .build_store(top_ptr.into_pointer_value(), first_param);

        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        self.builder
            .build_store(stack_size_addr, updated_stack_size);

        self.builder.build_return(None);
    }

    pub fn build_not(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let not_fn_type = void_type.fn_type(&[IntType(self.context.i64_type())], false);
        let not_fn = self.module.add_function("piet_not", not_fn_type, None);

        // Labels
        let basic_block = self.context.append_basic_block(not_fn, "");
        let then_block = self.context.append_basic_block(not_fn, "stack_nonempty");
        let ret_block = self.context.append_basic_block(not_fn, "ret");

        self.builder.position_at_end(basic_block);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_addr = self
            .module
            .get_global("piet_stack")
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

        let top_ptr = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };
        let top_ptr = self.builder.build_load(top_ptr, "top_elem_ptr");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr.into_pointer_value(), "top_elem_val")
            .into_int_value();

        let value_cmp = self.builder.build_int_compare(
            IntPredicate::EQ,
            top_ptr_val,
            const_0,
            "top_value_is_zero",
        );

        let zext_cmp =
            self.builder
                .build_int_z_extend(value_cmp, self.context.i64_type(), "zero_extend_cmp");
        self.builder
            .build_store(top_ptr.into_pointer_value(), zext_cmp);

        self.builder.build_unconditional_branch(ret_block);
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub fn build_dup(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let not_fn_type = void_type.fn_type(&[IntType(self.context.i64_type())], false);
        let not_fn = self.module.add_function("piet_dup", not_fn_type, None);

        // Labels
        let basic_block = self.context.append_basic_block(not_fn, "");
        let then_block = self.context.append_basic_block(not_fn, "stack_nonempty");
        let ret_block = self.context.append_basic_block(not_fn, "ret");

        self.builder.position_at_end(basic_block);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_0 = self.context.i64_type().const_int(0, false);
        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_addr = self
            .module
            .get_global("piet_stack")
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

        // Need to deref twice since LLVM globals are themselves pointers and we're operating on an i64**
        let top_ptr = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };
        let top_ptr = self.builder.build_load(top_ptr, "top_elem_ptr");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr.into_pointer_value(), "top_elem_val")
            .into_int_value();

        // Dup always increases stack size by 1
        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        let to_store = unsafe { self.builder.build_gep(stack_addr, &[stack_size_val], "") };

        // Push (dup) top element to stack
        self.builder.build_store(to_store, top_ptr_val);

        // Store updated stack size
        self.builder
            .build_store(stack_size_addr, updated_stack_size);

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub fn build_pop(&self) {
        // The stack is only valid from 0 to stack_size, so decrementing the stack size effectively pops the top element off the stack.
        let void_type = self.context.void_type();
        let pop_fn_type = void_type.fn_type(&[IntType(self.context.i64_type())], false);
        let pop_fn = self.module.add_function("piet_pop", pop_fn_type, None);

        // Labels
        let basic_block = self.context.append_basic_block(pop_fn, "");
        let then_block = self.context.append_basic_block(pop_fn, "stack_noempty");
        let ret_block = self.context.append_basic_block(pop_fn, "ret");

        self.builder.position_at_end(basic_block);

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let cmp = self.builder.build_int_compare(
            IntPredicate::SGE,
            stack_size_val,
            const_1,
            "check_stack_size",
        );

        self.builder
            .build_conditional_branch(cmp, then_block, ret_block);

        self.builder.position_at_end(then_block);

        let updated_stack_size =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size");

        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8).ok();
        self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

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
        let cont_block = self.context.append_basic_block(rotate_fn, "cont");
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

    pub fn build_input(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let in_fn_type = void_type.fn_type(&[], false);

        let in_fn = match instr {
            Instruction::IntIn => self.module.add_function("piet_intin", in_fn_type, None),
            Instruction::CharIn => self.module.add_function("piet_charin", in_fn_type, None),
            _ => panic!("Not an input instruction!"),
        };

        // Labels
        let basic_block = self.context.append_basic_block(in_fn, "");
        let ret_block = self.context.append_basic_block(in_fn, "ret");

        self.builder.position_at_end(basic_block);
        let read_addr = self
            .builder
            .build_alloca(self.context.i64_type(), "stack_alloc");

        let fmt = match instr {
            Instruction::IntIn => self
                .module
                .get_global("dec_fmt")
                .unwrap()
                .as_pointer_value(),
            Instruction::CharIn => self
                .module
                .get_global("char_fmt")
                .unwrap()
                .as_pointer_value(),
            _ => panic!("Not an output instruction"),
        };

        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);

        let const_fmt_gep = unsafe { self.builder.build_gep(fmt, &[const_0, const_0], "") };

        let scanf_fn = self.module.get_function("scanf").unwrap();
        let scanf =
            self.builder
                .build_call(scanf_fn, &[const_fmt_gep.into(), read_addr.into()], "scanf");

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

        let stack_size_val = self
            .builder
            .build_load(stack_size_addr, "stack_size")
            .into_int_value();

        let push_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[stack_size_val], "") };
        let push_ptr = self.builder.build_load(push_ptr_gep, "push_elem_ptr");

        let result = self.builder.build_load(read_addr, "scanf_elem");
        self.builder
            .build_store(push_ptr.into_pointer_value(), result);

        let updated_stack_size =
            self.builder
                .build_int_add(stack_size_val, const_1, "increment_stack_size");

        // Store updated stack size
        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8).ok();

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub fn build_output(&self, instr: Instruction) {
        let void_type = self.context.void_type();
        let out_fn_type = void_type.fn_type(&[], false);

        let out_fn = match instr {
            Instruction::IntOut => self.module.add_function("piet_intout", out_fn_type, None),
            Instruction::CharOut => self.module.add_function("piet_charout", out_fn_type, None),
            _ => panic!("Not an output instruction!"),
        };

        // Labels
        let basic_block = self.context.append_basic_block(out_fn, "");
        let then_block = self.context.append_basic_block(out_fn, "stack_nonempty");
        let ret_block = self.context.append_basic_block(out_fn, "ret");

        self.builder.position_at_end(basic_block);

        let const_0 = self.context.i64_type().const_zero();

        let stack_size_addr = self
            .module
            .get_global("stack_size")
            .unwrap()
            .as_pointer_value();

        let const_1 = self.context.i64_type().const_int(1, false);

        let stack_addr = self
            .module
            .get_global("piet_stack")
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

        // Need to deref twice since LLVM globals are themselves pointers and we're operating on an i64**
        let top_ptr_gep = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };
        let printf_fn = self.module.get_function("printf").unwrap();

        let fmt = match instr {
            Instruction::IntOut => self
                .module
                .get_global("dec_fmt")
                .unwrap()
                .as_pointer_value()
                .into(),
            Instruction::CharOut => self
                .module
                .get_global("char_fmt")
                .unwrap()
                .as_pointer_value()
                .into(),
            _ => panic!("Not an output instruction"),
        };

        let top_ptr_deref1 = self.builder.build_load(top_ptr_gep, "top_elem_deref1");

        let top_ptr_val = self
            .builder
            .build_load(top_ptr_deref1.into_pointer_value(), "top_elem_val");

        let const_fmt_gep = unsafe { self.builder.build_gep(fmt, &[const_0, const_0], "") };

        let printf = self.builder.build_call(
            printf_fn,
            &[const_fmt_gep.into(), top_ptr_val.into()],
            "printf",
        );

        let updated_stack_size =
            self.builder
                .build_int_sub(stack_size_val, const_1, "decrement_stack_size");

        let store = self
            .builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8).ok();
        self.builder.build_unconditional_branch(ret_block);

        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }

    pub fn build_binop(&self, instr: Instruction) {
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

    pub fn build_globals(&self) {
        // Our piet stack is a Vec<i64> so we want to malloc an i64
        let i64_ptr_type = self.context.i64_type().ptr_type(AddressSpace::default());
        let piet_stack = self.module.add_global(i64_ptr_type, None, "piet_stack");

        let i8_type = self.context.i8_type();
        let i64_type = self.context.i64_type();

        let init_dp = i8_type.const_int(0, false);
        let init_cc = i8_type.const_int(0, false);

        let global_dp = self.module.add_global(i8_type, None, "dp");
        let global_cc = self.module.add_global(i8_type, None, "cc");

        let global_stack_size = self.module.add_global(i64_type, None, "stack_size");

        // Defines dp, cc, and stack depth
        piet_stack.set_linkage(Linkage::Internal);
        piet_stack.set_initializer(&i64_ptr_type.const_null());

        global_dp.set_linkage(Linkage::Internal);
        global_dp.set_initializer(&init_dp);

        global_cc.set_linkage(Linkage::Internal);
        global_cc.set_initializer(&init_cc);

        global_stack_size.set_linkage(Linkage::Internal);
        global_stack_size.set_initializer(&i64_type.const_zero());

        // Initialize the piet stack
        let init_fn_type = self.context.void_type().fn_type(&[], false);
        let init_fn = self.module.add_function("init_globals", init_fn_type, None);
        let init_block = self.context.append_basic_block(init_fn, "");
        self.builder.position_at_end(init_block);

        unsafe {
            self.builder.build_global_string("%d\0", "dec_fmt");
            self.builder.build_global_string("%c\0", "char_fmt");
        }

        // malloc type
        let malloc_fn_type = i64_ptr_type.fn_type(&[self.context.i64_type().into()], false);
        let malloc_fn = self.module.add_function("malloc", malloc_fn_type, None);

        // printf type
        let c_string_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let printf_type = self
            .context
            .i64_type()
            .fn_type(&[c_string_type.into()], true);
        let printf_fn = self.module.add_function("printf", printf_type, None);

        // scanf type
        let scanf_type = self
            .context
            .i64_type()
            .fn_type(&[c_string_type.into()], true);

        let scanf_fn = self.module.add_function("scanf", scanf_type, None);

        let size_value = self.context.i64_type().const_int(STACK_SIZE as u64, false);
        let malloc_call = self
            .builder
            .build_call(malloc_fn, &[size_value.into()], "malloc");

        let value = malloc_call.try_as_basic_value().left().unwrap();
        self.builder
            .build_store(piet_stack.as_pointer_value(), value.into_pointer_value());

        self.builder.build_return(None);
    }

    fn build_main(&self) {
        let main_fn_type = self.context.i64_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);

        // Call init_globals
        let init_globals = self.module.get_function("init_globals").unwrap();
        let init_block = self.context.append_basic_block(main_fn, "");
        self.builder.position_at_end(init_block);

        self.builder.build_call(init_globals, &[], "setup_stack");
        self.builder
            .build_return(Some(&self.context.i64_type().const_zero()));
    }

    pub fn generate(&self) -> String {
        self.build_globals();
        /*self.build_binop(Instruction::Add);
        self.build_binop(Instruction::Sub);*/
        //self.build_binop(Instruction::Div);
        //self.build_binop(Instruction::Mul);
        //self.build_binop(Instruction::Mod);
        self.build_binop(Instruction::Mod);
        //self.build_push();
        //self.build_pop();
        //self.build_not();
        self.build_input(Instruction::CharIn);
        self.build_output(Instruction::CharOut);
        self.build_main();

        self.module.print_to_string().to_string()
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use inkwell::{builder::Builder, context::Context, module::Module};
    use types::program::Program;
    #[test]
    fn test_entrypoint() {
        let context = Context::create();
        let module = context.create_module("piet");
        let builder = context.create_builder();

        let vec = Vec::new();
        // Program
        let program = Program::new(&vec, 0, 0);
        let cfg_gen = CFGGenerator::new(&program, 1);
        let cg = CodeGen::new(&context, module, builder, cfg_gen);
        println!("{}", cg.generate())
    }
}
