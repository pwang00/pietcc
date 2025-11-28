use inkwell::{builder::Builder, context::Context, module::Module};
use parser::cfg::CFGBuilder;
use parser::decode::DecodeInstruction;
use piet_core::settings::CompilerSettings;

#[allow(unused)]
pub struct LoweringCtx<'a, 'b> {
    pub(crate) llvm_context: &'b Context,
    pub(crate) module: Module<'b>,
    pub(crate) builder: Builder<'b>,
    pub(crate) cfg_builder: CFGBuilder<'a>,
    pub(crate) settings: CompilerSettings<'a>,
}

impl<'a, 'b> DecodeInstruction for LoweringCtx<'a, 'b> {}

#[allow(unused)]
impl<'a, 'b> LoweringCtx<'a, 'b> {
    pub fn new(
        llvm_context: &'b Context,
        module: Module<'b>,
        builder: Builder<'b>,
        cfg_builder: CFGBuilder<'a>,
        settings: CompilerSettings<'a>,
    ) -> Self {
        Self {
            llvm_context,
            module,
            builder,
            cfg_builder,
            settings,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use inkwell::{builder::Builder, context::Context, module::Module};
    use parser::{convert::UnknownPixelSettings, loader::Loader};
    use piet_core::program::Program;
    use std::fs;

    const SETTINGS: UnknownPixelSettings = UnknownPixelSettings::TreatAsError;
    #[test]
    fn test_entrypoint() -> std::io::Result<()> {
        let context = Context::create();
        let module = context.create_module("piet");
        let builder = context.create_builder();
        // Program
        let program = Loader::convert("../images/alpha_filled.png", SETTINGS).unwrap();
        let cfg_gen = CFGBuilder::new(&program, 1, true);
        let mut cg = LoweringCtx::new(&context, module, builder, cfg_gen, 1);
        let options = SaveOptions::EmitLLVMIR;
        let ir = cg.build_all(
            "../../compilation.ll",
            OptimizationLevel::Aggressive,
            false,
            false,
            options,
        );
        Ok(())
    }
}
