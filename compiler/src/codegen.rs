use crate::cfg::CFGGenerator;
use crate::consts::STACK_SIZE;
use inkwell::{
    basic_block,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    AddressSpace,
};

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

    pub fn build_add(&self) {
        let void_type = self.context.void_type();
        let stack = self.module.get_global("piet_stack").unwrap();
        let add_fn_type = void_type.fn_type(&[], false);
        let add_fn = self.module.add_function("piet_add", add_fn_type, None);

        // i64s are 64 bits, so we want to do *(stack + stack_depth * 8) + *(stack + stack_depth * 8 - 8) if possible
        let basic_block = self.context.append_basic_block(add_fn, "");
        self.builder.position_at_end(basic_block);

        let stack_addr = self
            .module
            .get_global("piet_stack")
            .unwrap()
            .as_pointer_value();

        let stack_depth_addr = self
            .module
            .get_global("stack_depth")
            .unwrap()
            .as_pointer_value();
        
        let stack_depth = self
            .builder
            .build_load(stack_depth_addr, "load_depth")
            .into_int_value();

        let top_elem_ptr = self.builder.build_load(stack_addr, "top_elem");
        let next_elem_ptr = self.builder.build_load(
            unsafe { stack_addr.const_gep(&[self.context.i64_type().const_int(8, false)]) },
            "next_elem",
        );

        let top_elem_val = self.builder.build_load(top_elem_ptr.into_pointer_value(), "top_elem_val");
        let next_elem_val = self.builder.build_load(next_elem_ptr.into_pointer_value(), "next_elem_val");

        self.builder.build_int_add(top_elem_val.into_int_value(), next_elem_val.into_int_value(), "add");

        // TODO: Store

    }

    pub fn build_globals(&self) {
        // Our piet stack is a Vec<i64> so we want to malloc an i64
        let i64_ptr_type = self.context.i64_type().ptr_type(AddressSpace::default());
        let piet_stack = self.module.add_global(i64_ptr_type, None, "piet_stack");

        let dp_type = self.context.i8_type();
        let cc_type = self.context.i8_type();

        let init_dp = dp_type.const_int(0, false);
        let init_cc = cc_type.const_int(0, false);

        let global_dp = self.module.add_global(dp_type, None, "dp");
        let global_cc = self.module.add_global(cc_type, None, "cc");

        let stack_depth = self.context.i64_type();
        let stack_depth_val = stack_depth.const_int(0, false);

        let global_stack_depth = self.module.add_global(stack_depth, None, "stack_depth");

        // Defines dp, cc, and stack depth
        global_dp.set_linkage(Linkage::Internal);
        global_dp.set_initializer(&init_dp);

        global_cc.set_linkage(Linkage::Internal);
        global_cc.set_initializer(&init_cc);

        global_stack_depth.set_linkage(Linkage::Internal);
        global_stack_depth.set_initializer(&stack_depth_val);

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
        let main_fn_type = self.context.i32_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);

        // Call init_globals
        let init_globals = self.module.get_function("init_globals").unwrap();
        let init_block = self.context.append_basic_block(main_fn, "");
        self.builder.position_at_end(init_block);

        self.builder.build_call(init_globals, &[], "setup_stack");
    }

    pub fn generate(&self) -> String {
        self.build_globals();
        self.build_add();
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
        println!("{:?}", cg.generate())
    }
}
