use piet_core::cfg::{Node, CFG};
use piet_core::error::ExecutionError;
use piet_core::flow::find_offset;
use piet_core::instruction::*;
use piet_core::settings::{InterpreterSettings, Verbosity};
use piet_core::state::{ExecutionState, ExecutionStatus};
use std::collections::VecDeque;
use std::env;
use std::io::Write;
use std::{io, io::Read};

#[derive(Debug)]
pub struct Interpreter<'a> {
    cfg: &'a CFG,
    state: ExecutionState,
    settings: InterpreterSettings,
}

impl<'a> Interpreter<'a> {
    pub fn new(cfg: &'a CFG, settings: InterpreterSettings) -> Self {
        Self {
            cfg,
            state: ExecutionState::default(),
            settings,
        }
    }

    pub fn next_block(&mut self, block: Node) -> (Option<Node>, Option<Instruction>) {
        let bordering = &mut self.cfg.get(&block).unwrap();
        let mut directions = vec![];

        for adj in bordering.keys() {
            directions.extend(bordering.get(&*adj).unwrap().iter().map(|transition| {
                return (
                    transition.entry_state,
                    transition.exit_state,
                    adj,
                    transition.instruction,
                );
            }));
        }

        // Calculates the block who is the minimum amount of rotations away from the current entry direction
        // Want min exit conditioned on min entry
        let curr = self.state.pointers;
        match directions.into_iter().min_by_key(|&(entry, exit, _, _)| {
            let entry_offset = find_offset(curr, entry);
            let exit_offset = find_offset(entry, exit);
            (entry_offset, exit_offset)
        }) {
            Some((_, exit, adj, instr)) => {
                self.state.pointers = exit;
                (Some(adj.clone()), instr)
            }
            None => (None, None),
        }
    }

    pub fn get_state(&self) -> ExecutionState {
        self.state.clone()
    }

    pub fn is_complete(&self) -> ExecutionStatus {
        return self.state.status;
    }

    pub fn exec_instr(&mut self, instr: Instruction) -> Result<(), ExecutionError> {
        Ok(match instr {
            Instruction::Push => self.push(self.state.cb_count),
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
        self.state.stack.push_front(cb as i64)
    }

    #[inline]
    pub(crate) fn pop(&mut self) {
        self.state.stack.pop_front();
    }

    #[inline]
    pub(crate) fn add(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let b = self.state.stack.pop_front().unwrap();
            Ok(self.state.stack.push_front(a + b))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Add,
                format!(
                    "Skipping Add since Add requires at least 2 elements on stack but found {}",
                    self.state.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn sub(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let b = self.state.stack.pop_front().unwrap();
            Ok(self.state.stack.push_front(b - a))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Sub,
                format!(
                    "Skipping Sub since Sub requires at least 2 elements on stack but found {}",
                    self.state.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn mul(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let b = self.state.stack.pop_front().unwrap();
            Ok(self.state.stack.push_front(b * a))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Mul,
                format!(
                    "Skipping Mul since Mul requires at least 2 elements on stack but found {}",
                    self.state.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn div(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let b = self.state.stack.pop_front().unwrap();

            if a > 0 {
                Ok(self.state.stack.push_front(b / a))
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
                    self.state.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn rem(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let b = self.state.stack.pop_front().unwrap();

            if a > 0 {
                Ok(self.state.stack.push_front(b.rem_euclid(a)))
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
                    self.state.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn not(&mut self) -> Result<(), ExecutionError> {
        if let Some(a) = self.state.stack.pop_front() {
            Ok(self.state.stack.push_front(if a != 0 { 0 } else { 1 }))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Ptr,
                "Skipping Not since not requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn grt(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let b = self.state.stack.pop_front().unwrap();
            Ok(self.state.stack.push_front(if b > a { 1 } else { 0 }))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Gt,
                format!(
                    "Skipping Gt since Gt requires at least 2 elements on stack but found {}",
                    self.state.stack.len()
                ),
            ))
        }
    }

    pub(crate) fn ptr(&mut self) -> Result<(), ExecutionError> {
        if let Some(n) = self.state.stack.pop_front() {
            Ok(self.state.pointers.dp = self.state.pointers.dp.rotate(n))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Ptr,
                "Skipping Ptr since Ptr requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn swi(&mut self) -> Result<(), ExecutionError> {
        if let Some(n) = self.state.stack.pop_front() {
            Ok(self.state.pointers.cc = self.state.pointers.cc.switch(n))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Swi,
                "Skipping Swi since Swi requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn dup(&mut self) -> Result<(), ExecutionError> {
        if let Some(n) = self.state.stack.front() {
            Ok(self.state.stack.push_front(*n))
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Dup,
                "Skipping Dup since Dup requires at least 1 element on stack but found none".into(),
            ))
        }
    }

    pub(crate) fn roll(&mut self) -> Result<(), ExecutionError> {
        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop_front().unwrap();
            let n = self.state.stack.pop_front().unwrap();

            if n < 0 || n as usize > self.state.stack.len() {
                return Err(ExecutionError::StackOutOfBoundsError(
                    Instruction::Roll,
                    format!("Invalid value for n: {}", n),
                ));
            }

            let mut top_n = self
                .state
                .stack
                .range(0..n as usize)
                .map(|&x| x)
                .collect::<VecDeque<_>>();
            let rest = self.state.stack.range(n as usize..);
            if a < 0 {
                top_n.rotate_right((a.abs() % top_n.len() as i64) as usize);
            } else {
                top_n.rotate_left((a % top_n.len() as i64) as usize);
            }

            top_n.extend(rest);
            Ok(self.state.stack = top_n)
        } else {
            Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Roll,
                format!(
                    "Skipping Gt since Gt requires at least 2 elements on stack but found {}",
                    self.state.stack.len()
                ),
            ))
        }
    }

    #[inline]
    pub(crate) fn int_in(&mut self) -> Result<(), ExecutionError> {
        self.state.stdin.clear();
        if self.settings.abstract_interp {
            self.state.status = ExecutionStatus::NeedsInput;
            return Ok(());
        }
        io::stdin()
            .read_line(&mut self.state.stdin)
            .expect("Failed to read input");
        if let Ok(n) = self.state.stdin.trim().parse::<i64>() {
            Ok(self.state.stack.push_front(n))
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

        if self.settings.abstract_interp {
            self.state.status = ExecutionStatus::NeedsInput;
            return Ok(());
        }

        let char = io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as i64);

        if let Some(c) = char {
            Ok(self.state.stack.push_front(c))
        } else {
            Err(ExecutionError::ParseError(
                Instruction::IntIn,
                "Error parsing char input".into(),
            ))
        }
    }

    #[inline]
    pub(crate) fn int_out(&mut self) {
        if let Some(n) = self.state.stack.pop_front() {
            self.state.stdout.push(StdOutWrapper::Int(n));
            if self.settings.print {
                print!("{n}");
            }
        }
    }

    #[inline]
    pub(crate) fn char_out(&mut self) {
        if let Some(n) = self.state.stack.pop_front() {
            if let Some(c) = char::from_u32(n as u32) {
                self.state.stdout.push(StdOutWrapper::Char(c));
                if self.settings.print {
                    print!("{c}");
                }
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

    pub fn run(&mut self) -> ExecutionState {
        match self.settings.verbosity {
            Verbosity::Verbose => match env::consts::OS {
                "linux" => {
                    println!(
                        "\x1B[1;37mpietcc:\x1B[0m \x1B[1;96minfo: \x1B[0mrunning with {:?}",
                        self.settings.codel_settings
                    )
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
            self.state.cb_count = block.get_region().len() as u64;
            self.state.cb_label = block.get_label().clone();

            if let Some(max_steps) = self.settings.max_steps {
                if self.state.steps == max_steps {
                    self.state.status = ExecutionStatus::MaxSteps;
                    break;
                }
            }

            if self.settings.abstract_interp && self.state.status == ExecutionStatus::NeedsInput {
                break;
            }

            let (next, maybe_instr) = self.next_block(block.clone());

            if next.is_none() {
                self.state.status = ExecutionStatus::Completed;
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
        }

        self.state.clone()
    }
}

#[allow(unused)]
mod test {
    use super::*;
    use piet_core::color::Lightness;

    #[test]
    fn test_roll() {
        // Setup
        let vec = Vec::<Lightness>::new();
        let program = PietSource::new(&vec, 0, 0);
        let mut interpreter = Interpreter::new(&program, InterpreterSettings::default());

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
