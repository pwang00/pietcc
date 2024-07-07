use crate::settings::{InterpSettings, Verbosity};
use parser::cfg::{Node, CFG};
use std::collections::VecDeque;
use std::env;
use std::io::Write;
use std::{io, io::Read};
use types::error::ExecutionError;
use types::flow::find_offset;
use types::instruction::Instruction;
use types::state::{ExecutionResult, ExecutionState};

pub struct Interpreter {
    cfg: CFG,
    state: ExecutionState,
    stack: VecDeque<i64>,
    stdout_instrs: Vec<(Instruction, i64)>,
    settings: InterpSettings,
}

impl Interpreter {
    pub fn new(cfg: CFG, settings: InterpSettings) -> Self {
        Self {
            cfg,
            state: ExecutionState::default(),
            stack: VecDeque::new(),
            stdout_instrs: Vec::new(),
            settings,
        }
    }

    pub fn next_block(&mut self, block: Node) -> (Option<Node>, Option<Instruction>) {
        let bordering = &mut self.cfg.get(&block).unwrap();
        let mut directions = vec![];

        for adj in bordering.keys() {
            directions.extend(
                bordering
                    .get(&*adj)
                    .unwrap()
                    .iter()
                    .map(|(entry, exit, instr)| {
                        return (entry, exit, adj, instr);
                    }),
            );
        }

        // Calculates the block who is the minimum amount of rotations away from the current entry direction
        // Want min exit conditioned on min entry
        let curr = (self.state.dp, self.state.cc);
        match directions.into_iter().min_by_key(|&(&entry, &exit, _, _)| {
            let entry_offset = find_offset(curr, entry);
            let exit_offset = find_offset(entry, exit);
            (entry_offset, exit_offset)
        }) {
            Some((_, &exit, adj, &instr)) => {
                (self.state.dp, self.state.cc) = exit;
                (Some(adj.clone()), instr)
            }
            None => (None, None),
        }
    }

    pub fn get_state(&self) -> ExecutionState {
        self.state.clone()
    }

    pub fn get_stack(&self) -> VecDeque<i64> {
        self.stack.clone()
    }

    pub fn get_stdout_instrs(&self) -> Vec<(Instruction, i64)> {
        self.stdout_instrs.clone()
    }

    pub fn exec_instr(&mut self, instr: Instruction) -> Result<(), ExecutionError> {
        Ok(match instr {
            Instruction::Push => self.push(self.state.cb),
            Instruction::Pop => self.pop(),
            Instruction::Add => self.add()?,
            Instruction::Sub => self.sub()?,
            Instruction::Mul => self.mul()?,
            Instruction::Div => self.div()?,
            Instruction::Mod => self.rem()?,
            Instruction::Not => self.not()?,
            Instruction::Gt => self.grt()?,
            Instruction::Ptr => self.ptr()?,
            Instruction::Swi => self.swi()?,
            Instruction::Dup => self.dup()?,
            Instruction::Roll => self.roll()?,
            Instruction::CharIn => self.char_in()?,
            Instruction::CharOut => self.char_out(),
            Instruction::IntIn => self.int_in()?,
            Instruction::IntOut => self.int_out(),
        })
    }

    #[inline]
    pub(crate) fn push(&mut self, cb: u64) {
        self.stack.push_front(cb as i64)
    }

    #[inline]
    pub(crate) fn pop(&mut self) {
        self.stack.pop_front();
    }

    #[inline]
    pub(crate) fn add(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let b = self.stack.pop_front().unwrap();
            Ok(self.stack.push_front(a + b))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Add,
                format!(
                    "Skipping Add since Add requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn sub(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let b = self.stack.pop_front().unwrap();
            Ok(self.stack.push_front(b - a))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Sub,
                format!(
                    "Skipping Sub since Sub requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn mul(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let b = self.stack.pop_front().unwrap();
            Ok(self.stack.push_front(b * a))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Mul,
                format!(
                    "Skipping Mul since Mul requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn div(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let b = self.stack.pop_front().unwrap();

            if a > 0 {
                Ok(self.stack.push_front(b / a))
            } else {
                Err(ExecutionError::DivisionByZeroError(
                    Instruction::Div,
                    format!("Attempted to divide {b} by 0"),
                ))
            }
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Div,
                format!(
                    "Skipping Div since Div requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn rem(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let b = self.stack.pop_front().unwrap();

            if a > 0 {
                Ok(self.stack.push_front(b.rem_euclid(a)))
            } else {
                Err(ExecutionError::DivisionByZeroError(
                    Instruction::Mod,
                    format!("Attempted to divide {b} by 0"),
                ))
            }
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Mod,
                format!(
                    "Skipping Mod since Mod requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn not(&mut self) -> Result<(), ExecutionError> {
        if let Some(a) = self.stack.pop_front() {
            Ok(self.stack.push_front(if a != 0 { 0 } else { 1 }))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Ptr,
                "Skipping Not since not requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn grt(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let b = self.stack.pop_front().unwrap();
            Ok(self.stack.push_front(if b > a { 1 } else { 0 }))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Gt,
                format!(
                    "Skipping Gt since Gt requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    pub(crate) fn ptr(&mut self) -> Result<(), ExecutionError> {
        if let Some(n) = self.stack.pop_front() {
            Ok(self.state.dp = self.state.dp.rotate(n))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Ptr,
                "Skipping Ptr since Ptr requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn swi(&mut self) -> Result<(), ExecutionError> {
        if let Some(n) = self.stack.pop_front() {
            Ok(self.state.cc = self.state.cc.switch(n))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Swi,
                "Skipping Swi since Swi requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn dup(&mut self) -> Result<(), ExecutionError> {
        if let Some(n) = self.stack.front() {
            Ok(self.stack.push_front(*n))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Dup,
                "Skipping Dup since Dup requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn roll(&mut self) -> Result<(), ExecutionError> {
        if self.stack.len() >= 2 {
            let a = self.stack.pop_front().unwrap();
            let n = self.stack.pop_front().unwrap();

            if n < 0 || n as usize > self.stack.len() {
                return Err(ExecutionError::StackOutOfBoundsError(
                    Instruction::Roll,
                    format!("Invalid value for n: {}", n),
                ));
            }

            let mut top_n = self
                .stack
                .range(0..n as usize)
                .map(|&x| x)
                .collect::<VecDeque<_>>();
            let rest = self.stack.range(n as usize..);
            if a < 0 {
                top_n.rotate_right((a.abs() % top_n.len() as i64) as usize);
            } else {
                top_n.rotate_left((a % top_n.len() as i64) as usize);
            }

            top_n.extend(rest);
            Ok(self.stack = top_n)
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Roll,
                format!(
                    "Skipping Gt since Gt requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn int_in(&mut self) -> Result<(), ExecutionError> {
        self.state.stdin.clear();
        let _ = io::stdin().read_line(&mut self.state.stdin);
        if let Ok(n) = self.state.stdin.trim().parse::<i64>() {
            Ok(self.stack.push_front(n))
        } else {
            Err(ExecutionError::ParseError(
                Instruction::IntIn,
                "Error parsing int input".into(),
            ))
        }
    }

    #[inline]
    pub(crate) fn char_in(&mut self) -> Result<(), ExecutionError> {
        self.state.stdin.clear();
        let char = io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as i64);

        if let Some(c) = char {
            Ok(self.stack.push_front(c))
        } else {
            Err(ExecutionError::ParseError(
                Instruction::IntIn,
                "Error parsing char input".into(),
            ))
        }
    }

    #[inline]
    pub(crate) fn int_out(&mut self) {
        if let Some(n) = self.stack.pop_front() {
            print!("{n}");
        }
    }

    #[inline]
    pub(crate) fn char_out(&mut self) {
        if let Some(n) = self.stack.pop_front() {
            if let Some(c) = char::from_u32(n as u32) {
                print!("{c}");
            }
        }
    }

    pub fn get_entry(&self) -> Node {
        self.cfg
            .keys()
            .find(|node| *node.get_label() == "Entry")
            .unwrap()
            .clone()
    }

    pub fn run(&mut self) -> ExecutionResult {
        match self.settings.verbosity {
            Verbosity::Verbose => match env::consts::OS {
                "linux" => {
                    println!("\x1B[1;37mpietcc:\x1B[0m \x1B[1;96minfo: \x1B[0mrunning with {:?}",
                        self.settings.codel_settings)
                }
                _ => {
                    println!(
                        "pietcc: info: running with {:?}",
                        self.settings.codel_settings
                    )
                }
            },
            _ => (),
        }

        let mut block = self.get_entry();

        loop {
            io::stdout().flush().unwrap();
            self.state.cb = block.get_region().len() as u64;
            let (next, maybe_instr) = self.next_block(block.clone());

            if next.is_none() {
                break;
            }

            block = next.unwrap();

            if let Some(instr) = maybe_instr {
                let res = self.exec_instr(instr);
                if let Err(res) = res {
                    if self.settings.verbosity == Verbosity::Verbose {
                        eprintln!("{:?}", res);
                    }
                }
                self.state.steps += 1;
            }

            if let Some(max_steps) = self.settings.max_steps {
                if self.state.steps == max_steps {
                    break;
                }
            }
        }

        ExecutionResult {
            state: &self.state,
            stack: &self.stack,
            stdout: &self.stdout_instrs,
        }
    }
}

#[allow(unused)]
mod test {
    use super::*;
    use types::color::Lightness;

    #[test]
    fn test_roll() {
        // Setup
        let vec = Vec::<Lightness>::new();
        let program = PietSource::new(&vec, 0, 0);
        let mut interpreter = Interpreter::new(&program, InterpSettings::default());

        // Positive roll to depth 2
        interpreter.stack = VecDeque::from([1, 2, 6, 5]);
        interpreter.roll();
        assert_eq!(interpreter.stack.pop_front(), Some(5));
        assert_eq!(interpreter.stack.pop_front(), Some(6));

        // Negative roll to depth 3
        interpreter.stack = VecDeque::from([-1, 3, 6, 5, 4]);
        interpreter.roll();
        assert_eq!(interpreter.stack.pop_front(), Some(4));
        assert_eq!(interpreter.stack.pop_front(), Some(6));
        assert_eq!(interpreter.stack.pop_front(), Some(5));

        // Negative roll to depth 2
        interpreter.stack = VecDeque::from([-1, 2, 6, 5, 4]);
        interpreter.roll();
        assert_eq!(interpreter.stack.pop_front(), Some(5));
        assert_eq!(interpreter.stack.pop_front(), Some(6));
        assert_eq!(interpreter.stack.pop_front(), Some(4));

        interpreter.stack = VecDeque::from([1, 3, 6, 5, 4]);
        interpreter.roll();
        assert_eq!(interpreter.stack.pop_front(), Some(5));
        assert_eq!(interpreter.stack.pop_front(), Some(4));
        assert_eq!(interpreter.stack.pop_front(), Some(6));

        interpreter.stack = VecDeque::from([-1, 2, 6, 5, 4]);
        interpreter.roll();
        assert_eq!(interpreter.stack.pop_front(), Some(5));
        assert_eq!(interpreter.stack.pop_front(), Some(6));
        assert_eq!(interpreter.stack.pop_front(), Some(4));
    }
}
