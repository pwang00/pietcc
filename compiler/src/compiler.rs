use crate::{cfg::CFGGenerator, codegen::CodeGen};
pub struct Compiler<'a> {
    cfg: CFGGenerator<'a>,
    codegen: CodeGen<'a>,
}
