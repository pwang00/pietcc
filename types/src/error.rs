use crate::instruction::Instruction;

#[derive(Debug)]
pub enum ExecutionError {
    ParseError(Instruction, String),
    StackOutOfBoundsError(Instruction, String),
    DivisionByZeroError(Instruction, String),
}

#[derive(Debug)]
pub struct InvalidPixelError(pub [u8; 3]);
