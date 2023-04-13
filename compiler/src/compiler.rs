use crate::{cfg::CFGGenerator, codegen::CodeGen};
pub struct Compiler<'a, 'b> {
    cfg: CFGGenerator<'a>,
    codegen: CodeGen<'a, 'b>,
    codel_size: u32,
}
