use crate::codegen::CodeGen;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_roll(&self) {
        let void_type = self.context.void_type();
        let roll_fn_type = void_type.fn_type(&[], false);
        let roll_fn = self.module.add_function("piet_roll", roll_fn_type, None);

        let const_0 = self.context.i64_type().const_zero();
        let const_1 = self.context.i64_type().const_int(1, false);
        let const_4 = self.context.i64_type().const_int(4, false);

        let basic_block = self.context.append_basic_block(roll_fn, "");
        self.builder.position_at_end(basic_block);
        let then_block = self.context.append_basic_block(roll_fn, "");
        let else_block = self.context.append_basic_block(roll_fn, "");
        let ret_block = self.context.insert_basic_block_after(else_block, "ret");
    }
}
