use inkwell::types::BasicTypeEnum;
use inkwell::AddressSpace;
use crate::codegen::CodeGen;
// ...

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_stdout_unbuffered(&self) {
        let void_type = self.context.void_type().fn_type(&[], false);
        let set_stdout_unbuffered_fn = self.module.add_function("set_stdout_unbuffered", void_type, None);

        // Basic blocks
        let basic_block = self.context.append_basic_block(set_stdout_unbuffered_fn, "");
        // Constants
        let const_0 = self.context.i32_type().const_zero();
        let setvbuf_fn = self.module.get_function("setvbuf").unwrap();
        let fdopen_fn = self.module.get_function("fdopen").unwrap();
        let stdout_fd = self.context.i32_type().const_int(1, false); // File descriptor for stdout is 1
        let fdopen_mode = self.module.get_global("fdopen_mode").unwrap().as_pointer_value();
        let fdopen_fmt_gep = unsafe { self.builder.build_gep(fdopen_mode, &[const_0, const_0], "") };
        self.builder.position_at_end(basic_block);

        let stdout_file_ptr = self.builder.build_call(fdopen_fn, &[stdout_fd.into(), fdopen_fmt_gep.into()], "stdout")
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();

        let null_ptr = self.context.i8_type().ptr_type(AddressSpace::default()).const_null();
        let unbuffered = self.context.i32_type().const_int(2, false);
        
        self.builder.build_call(
                setvbuf_fn,
                &[
                    stdout_file_ptr.into(),
                    null_ptr.into(),
                    unbuffered.into(),
                    const_0.into(),
                ],
                "",
            );
        
        self.builder.build_return(None);
    }
}

