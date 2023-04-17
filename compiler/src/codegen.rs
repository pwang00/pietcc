use crate::cfg::CFGGenerator;
use inkwell::{builder::Builder, context::Context, module::Module};

use types::instruction::Instruction;

#[allow(unused)]
pub struct CodeGen<'a, 'b> {
    pub(crate) context: &'b Context,
    pub(crate) module: Module<'b>,
    pub(crate) builder: Builder<'b>,
    pub(crate) cfg: CFGGenerator<'a>,
}

#[allow(unused)]
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

    pub fn generate(&self) -> String {
        self.build_globals();
        //self.build_binops(Instruction::Add);
        //self.build_binop(Instruction::Sub);
        //self.build_binop(Instruction::Div);
        //self.build_binop(Instruction::Mul);
        self.build_binops(Instruction::Mod);
        self.build_dup();
        self.build_push();
        //self.build_pop();
        //self.build_not();
        self.build_print_stack();
        self.build_input(Instruction::IntIn);
        self.build_output(Instruction::IntOut);
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
