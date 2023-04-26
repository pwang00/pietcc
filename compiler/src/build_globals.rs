use crate::{codegen::CodeGen, consts::STACK_SIZE};
use inkwell::{module::Linkage, AddressSpace};
use strum::IntoEnumIterator;
use types::instruction::Instruction;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_globals(&self) {
        // Our piet stack is a Vec<i64> so we want to malloc an i64
        let i64_ptr_type = self.context.i64_type().ptr_type(AddressSpace::default());
        let piet_stack = self.module.add_global(i64_ptr_type, None, "piet_stack");

        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();

        let init_dp = i8_type.const_int(0, false);
        let init_cc = i8_type.const_int(0, false);

        let global_dp = self.module.add_global(i8_type, None, "dp");
        let global_cc = self.module.add_global(i8_type, None, "cc");

        let global_stack_size = self.module.add_global(i64_type, None, "stack_size");
        let global_retries = self.module.add_global(i8_type, None, "rctr");
        // Defines dp, cc, and stack depth
        piet_stack.set_linkage(Linkage::Internal);
        piet_stack.set_initializer(&i64_ptr_type.const_null());

        global_dp.set_linkage(Linkage::Internal);
        global_dp.set_initializer(&init_dp);

        global_cc.set_linkage(Linkage::Internal);
        global_cc.set_initializer(&init_cc);

        global_stack_size.set_linkage(Linkage::Internal);
        global_stack_size.set_initializer(&i64_type.const_zero());

        global_retries.set_linkage(Linkage::Internal);
        global_retries.set_initializer(&i8_type.const_zero());

        // Initialize the piet stack
        let init_fn_type = self.context.void_type().fn_type(&[], false);
        let init_fn = self.module.add_function("init_globals", init_fn_type, None);
        let init_block = self.context.append_basic_block(init_fn, "");
        self.builder.position_at_end(init_block);

        unsafe {
            self.builder
                .build_global_string("Enter number: ", "input_message_int");
            self.builder
                .build_global_string("Enter char: ", "input_message_char");
            self.builder.build_global_string("%ld\0", "dec_fmt");
            self.builder.build_global_string("%c\0", "char_fmt");
            self.builder.build_global_string("%ld \0", "stack_fmt");
            self.builder
                .build_global_string("dp: %d, cc: %d\n\0", "ptr_fmt");
            self.builder
                .build_global_string("\nStack (size %d): ", "stack_id");
            self.builder
                .build_global_string("\nStack empty", "stack_id_empty");

            self.builder.build_global_string("w", "fdopen_mode");
            for instr in Instruction::iter() {
                self.builder.build_global_string(
                    &(instr.to_llvm_name().to_owned() + "\n"),
                    &(instr.to_llvm_name().to_owned() + "_fmt"),
                );
            }
            self.builder
                .build_global_string("Calling retry", "retry_fmt");
            self.builder.build_global_string("\n", "newline");
        }

        /* External functions and LLVM intrinsics */

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

        let _scanf_fn = self.module.add_function("__isoc99_scanf", scanf_type, None);

        let size_value = self.context.i64_type().const_int(STACK_SIZE as u64, false);
        let malloc_call = self
            .builder
            .build_call(malloc_fn, &[size_value.into()], "malloc");

        let value = malloc_call.try_as_basic_value().left().unwrap();
        self.builder
            .build_store(piet_stack.as_pointer_value(), value.into_pointer_value());

        let llvm_stackrestore_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        let _llvm_stackrestore_fn =
            self.module
                .add_function("llvm.stackrestore", llvm_stackrestore_type, None);

        let llvm_smax_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let _llvm_smax_fn = self
            .module
            .add_function("llvm.smax.i64", llvm_smax_type, None);

        let llvm_stacksave_type = i8_ptr_type.fn_type(&[], false);
        let _llvm_stacksave_fn =
            self.module
                .add_function("llvm.stacksave", llvm_stacksave_type, None);

        let exit_fn_type = void_type.fn_type(&[i64_type.into()], false);
        let _llvm_stacksave_fn = self.module.add_function("exit", exit_fn_type, None);

        // setvbuf to disable buffering
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        let setvbuf_type = void_type.fn_type(
            &[
                i8_ptr_type.into(),
                i8_ptr_type.into(),
                i32_type.into(),
                i32_type.into(),
            ],
            false,
        );

        self.module.add_function("setvbuf", setvbuf_type, None);

        // fdopen to get pointer to stdout
        let fdopen_type = i8_ptr_type.fn_type(&[i32_type.into(), c_string_type.into()], false);
        let fdopen_fn = self.module.add_function("fdopen", fdopen_type, None);

        self.builder.build_return(None);
    }
}
