use inkwell::{AddressSpace, module::Linkage};
use crate::{codegen::CodeGen, consts::STACK_SIZE};

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_globals(&self) {
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
        let _printf_fn = self.module.add_function("printf", printf_type, None);

        // scanf type
        let scanf_type = self
            .context
            .i64_type()
            .fn_type(&[c_string_type.into()], true);

        let _scanf_fn = self.module.add_function("scanf", scanf_type, None);

        let size_value = self.context.i64_type().const_int(STACK_SIZE as u64, false);
        let malloc_call = self
            .builder
            .build_call(malloc_fn, &[size_value.into()], "malloc");

        let value = malloc_call.try_as_basic_value().left().unwrap();
        self.builder
            .build_store(piet_stack.as_pointer_value(), value.into_pointer_value());

        self.builder.build_return(None);
    }
}