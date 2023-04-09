use crate::settings::{CodelSettings, InterpSettings, Verbosity};
use parser::infer::InferCodelWidth;
use parser::{convert::ConvertToLightness, decode::DecodeInstruction};
use std::collections::{HashSet, VecDeque};
use std::{io, io::Read};
use types::color::Lightness::Black;
use types::error::ExecutionError;
use types::flow::{Codel, Direction, FindAdj, FURTHEST, MOVE_IN};
use types::instruction::Instruction;
use types::program::Program;
use types::state::{ExecutionResult, ExecutionState, Position};

pub struct Interpreter<'a> {
    program: &'a Program<'a>,
    state: ExecutionState,
    stack: VecDeque<i64>,
    codel_width: u32,
    settings: InterpSettings,
}

impl<'a> DecodeInstruction for Interpreter<'a> {}
impl<'a> ConvertToLightness for Interpreter<'a> {}
impl<'a> FindAdj for Interpreter<'a> {}
impl<'a> InferCodelWidth for Interpreter<'a> {}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a Program, settings: InterpSettings) -> Self {
        let codel_width = match settings.codel_settings {
            CodelSettings::Default => 1,
            CodelSettings::Infer => Self::infer_codel_width(&program),
            CodelSettings::Width(codel_width) => codel_width,
        };

        Self {
            program,
            state: ExecutionState::default(),
            stack: VecDeque::new(),
            codel_width,
            settings,
        }
    }

    pub(crate) fn furthest_in_direction(&mut self, region: &HashSet<Position>) -> Position {
        let idx = match (self.state.dp, self.state.cc) {
            (Direction::Right, Codel::Left) => 0,
            (Direction::Right, Codel::Right) => 1,
            (Direction::Down, Codel::Left) => 2,
            (Direction::Down, Codel::Right) => 3,
            (Direction::Left, Codel::Left) => 4,
            (Direction::Left, Codel::Right) => 5,
            (Direction::Up, Codel::Left) => 6,
            (Direction::Up, Codel::Right) => 7,
        };

        *region.iter().max_by_key(FURTHEST[idx]).unwrap()
    }

    pub(crate) fn move_to_next_block(&self, (x, y): Position) -> Position {
        Some((x, y, self.codel_width))
            .map(MOVE_IN[self.state.dp as usize])
            .unwrap()
    }

    pub(crate) fn explore_region(&self, entry: Position) -> HashSet<Position> {
        let mut queue = VecDeque::from([entry]);
        let mut discovered = HashSet::new();
        let lightness = *self.program.get(entry).unwrap();

        while !queue.is_empty() {
            let pos = queue.pop_front().unwrap();
            discovered.insert(pos);

            let adjs = self.adjacencies(pos, self.program, self.codel_width);
            let in_block = adjs
                .iter()
                .filter(|&&pos| *self.program.get(pos).unwrap() == lightness)
                .collect::<Vec<_>>();

            for adj in in_block {
                if !discovered.contains(adj) {
                    queue.push_back(*adj);
                }
                discovered.insert(*adj);
            }
        }
        discovered
    }

    pub(crate) fn recalculate_entry(&mut self, region: &HashSet<Position>) -> Position {
        if self.state.rctr % 2 == 1 {
            self.state.dp = self.state.dp.rotate(1);
        } else {
            self.state.cc = self.state.cc.switch(1);
        }

        self.furthest_in_direction(region)
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
            return Err(ExecutionError::StackOutOfBoundsError(
                Instruction::Roll,
                format!(
                    "Skipping Gt since Gt requires at least 2 elements on stack but found {}",
                    self.stack.len()
                ),
            ));
        }
    }

    #[inline]
    pub(crate) fn int_in(&mut self) -> Result<(), ExecutionError> {
        let _ = io::stdin().read_line(&mut self.state.stdin);
        if let Ok(n) = self.state.stdin.parse::<i64>() {
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
            print!("{}", char::from_u32(n as u32).unwrap());
        }
    }

    pub fn run(&mut self) -> ExecutionResult {
        match self.settings.verbosity {
            Verbosity::Normal | Verbosity::Verbose => println!(
                "Running with codel width of {} (size of {})\n",
                self.codel_width,
                self.codel_width.pow(2)
            ),
            _ => (),
        }

        // A Piet program terminates when the retries counter reaches 8
        while self.state.rctr < 8 {
            let block = self.explore_region(self.state.pos);
            let lightness = *self.program.get(self.state.pos).unwrap();
            let furthest_in_dir = self.furthest_in_direction(&block);
            let next_pos = self.move_to_next_block(furthest_in_dir);
            let adj_lightness = self.program.get(next_pos);
            self.state.cb = block.len() as u64;

            match adj_lightness {
                None | Some(Black) => {
                    self.state.pos = self.recalculate_entry(&block);
                    self.state.rctr += 1;
                }
                Some(&color) => {
                    self.state.pos = next_pos;

                    if let Some(instr) = <Self as DecodeInstruction>::decode_instr(lightness, color)
                    {
                        let res = self.exec_instr(instr);

                        if let Err(res) = res {
                            if self.settings.verbosity == Verbosity::Verbose {
                                eprintln!("{:?}", res);
                            }
                        }
                    }

                    self.state.rctr = 0;
                }
            };

            self.state.steps += 1;
        }

        ExecutionResult {
            state: &self.state,
            stack: &self.stack,
        }
    }
}

mod test {
    use super::*;
    use types::color::Lightness;

    #[test]
    fn test_roll() {
        // Setup
        let vec = Vec::<Lightness>::new();
        let program = Program::new(&vec, 0, 0);
        let mut interpreter = Interpreter::new(&program, true);

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
    }
}
