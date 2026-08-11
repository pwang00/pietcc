use inkwell::{builder::Builder, context::Context, module::Module};
use piet_core::settings::CompilerSettings;

#[allow(unused)]
pub struct LoweringCtx<'a, 'b> {
    pub(crate) llvm_context: &'b Context,
    pub(crate) module: Module<'b>,
    pub(crate) builder: Builder<'b>,
    pub(crate) settings: CompilerSettings<'a>,
}

#[allow(unused)]
impl<'a, 'b> LoweringCtx<'a, 'b> {
    pub fn new(
        llvm_context: &'b Context,
        module: Module<'b>,
        builder: Builder<'b>,
        settings: CompilerSettings<'a>,
    ) -> Self {
        Self {
            llvm_context,
            module,
            builder,
            settings,
        }
    }
}
