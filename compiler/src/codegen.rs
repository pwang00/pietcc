use crate::cfg::CFGGenerator;
use crate::consts::STACK_SIZE;
use inkwell::{
    basic_block,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::BasicMetadataTypeEnum::IntType,
    types::BasicType,
    values::{BasicValue, FunctionValue, IntValue},
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

        self.builder.build_conditional_branch(stack_size_cmp, then_block, ret_block);
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

        let zext_cmp = self.builder.build_int_z_extend(value_cmp, self.context.i64_type(), "zero_extend_cmp");
        self.builder
            .build_store(top_ptr.into_pointer_value(), zext_cmp);

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

        let updated_stack_size = self.builder.build_int_sub(stack_size_val, const_1, "decrement_stack_size");
        
        let store = self.builder
            .build_store(stack_size_addr, updated_stack_size);

        store.set_alignment(8);
        
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
        let basic_block = self.context.append_basic_block(binop_fn, "");
        self.builder.position_at_end(basic_block);
        let cont_block = self.context.append_basic_block(binop_fn, "cont");

        // These are only used for modulo
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

        let top_ptr = unsafe { self.builder.build_gep(stack_addr, &[top_idx], "") };

        let next_idx = self
            .builder
            .build_int_sub(stack_size_val, const_2, "next_elem_idx");

        let next_ptr = unsafe { self.builder.build_gep(stack_addr, &[next_idx], "") };

        let top_ptr = self.builder.build_load(top_ptr, "top_elem_ptr");
        let next_ptr = self.builder.build_load(next_ptr, "next_elem_ptr");
        
        let top_ptr_val = self
            .builder
            .build_load(top_ptr.into_pointer_value(), "top_elem_val");

        let next_ptr_val = self
            .builder
            .build_load(next_ptr.into_pointer_value(), "next_elem_val");

        let result = match instr {
            Instruction::Add => self.builder.build_int_add(
                next_ptr_val.into_int_value(),
                top_ptr_val.into_int_value(),
                "add",
            ),
            Instruction::Sub => self.builder.build_int_sub(
                next_ptr_val.into_int_value(),
                top_ptr_val.into_int_value(),
                "sub",
            ),
            Instruction::Mul => self.builder.build_int_mul(
                next_ptr_val.into_int_value(),
                top_ptr_val.into_int_value(),
                "mul",
            ),
            Instruction::Div => {
                let cmp = self.builder.build_int_compare(
                    IntPredicate::NE,
                    top_ptr_val.into_int_value(),
                    const_0,
                    "check_dividend_nonzero",
                );
                self.builder
                    .build_conditional_branch(cmp, then_block, ret_block);

                then_block.set_name("dividend_nonzero");
                self.builder.position_at_end(then_block);
                self.builder.build_int_signed_div(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "div",
                )
            }
            Instruction::Mod => {
                let mut rem = self.builder.build_int_signed_rem(
                    next_ptr_val.into_int_value(),
                    top_ptr_val.into_int_value(),
                    "mod",
                );

                /* Modulo is just
                   if remainder > 0 then remainder
                   else modulus + remainder
                */
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
                rem = self
                    .builder
                    .build_int_add(top_ptr_val.into_int_value(), rem, "rem_result");
                self.builder.position_at_end(else_block);
                rem
            }
            Instruction::Gt => {
                /* Modulo is just
                   if remainder > 0 then remainder
                   else modulus + remainder
                */
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
                self.builder.build_int_z_extend(value_cmp, self.context.i64_type(), "zero_extend_cmp")
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

        let malloc_fn_type = i64_ptr_type.fn_type(&[self.context.i64_type().into()], false);
        let malloc_fn = self.module.add_function("malloc", malloc_fn_type, None);

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
        self.build_binop(Instruction::Gt);
        self.build_push();
        self.build_pop();
        self.build_not();
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
