use strum_macros::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Eq, Copy, Clone)]
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

#[derive(Copy, Clone, Debug)]
pub enum StdOutWrapper {
    Char(char),
    Int(i64),
}

impl Instruction {
    pub fn to_llvm_name(&self) -> &'static str {
        match self {
            Self::Push => "piet_push",
            Self::Pop => "piet_pop",
            Self::Add => "piet_add",
            Self::Sub => "piet_sub",
            Self::Mul => "piet_mul",
            Self::Div => "piet_div",
            Self::Mod => "piet_mod",
            Self::Not => "piet_not",
            Self::Gt => "piet_gt",
            Self::Ptr => "piet_rotate",
            Self::Swi => "piet_switch",
            Self::Dup => "piet_dup",
            Self::Roll => "piet_roll",
            Self::CharIn => "piet_charin",
            Self::CharOut => "piet_charout",
            Self::IntIn => "piet_intin",
            Self::IntOut => "piet_intout",
        }
    }
}
