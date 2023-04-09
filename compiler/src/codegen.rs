use crate::cfg::CFGGenerator;
use inkwell::{builder::Builder, context::Context, module::Module};

pub struct CodeGen<'a> {
    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
    cfg: &'a CFGGenerator<'a>,
}

impl<'a> CodeGen<'a> {
    pub fn new(
        context: &'a Context,
        module: Module<'a>,
        builder: Builder<'a>,
        cfg: &'a CFGGenerator<'a>,
    ) -> Self {
        CodeGen {
            context: &context,
            module,
            builder,
            cfg: &cfg,
        }
    }

    fn build_entrypoint(&self) {
        let i64_type = self.context.i64_type();
        self.builder.build_malloc(self.context.i64_type(), "malloc");
    }
}
