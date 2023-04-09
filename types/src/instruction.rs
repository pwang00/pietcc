#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Instruction {
    Push,
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    Gt,
    Ptr,
    Swi,
    Dup,
    Roll,
    CharIn,
    CharOut,
    IntIn,
    IntOut,
}
