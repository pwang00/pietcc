use crate::instruction::Instruction;

#[derive(Debug)]
pub enum ExecutionError {
    ParseError(Instruction, String),
    StackOutOfBoundsError(Instruction, String),
    DivisionByZeroError(Instruction, String),
}