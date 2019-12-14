use std::convert::TryInto;
use std::io;
use std::io::{BufRead, BufReader};
use std::num;

#[derive(Debug)]
pub enum Error {
    IndexOutOfBounds(usize),
    PcOutOfBounds(i64),
    UnknownOpcode { pc: usize, opcode: i64 },
    ReadIoError(io::Error),
    ParseIoError(num::ParseIntError),
    InvalidParameterMode { index: usize },
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::ReadIoError(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_error: std::str::Utf8Error) -> Self {
        Error::ReadIoError(io::Error::from(io::ErrorKind::InvalidData))
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Error::ParseIoError(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy)]
enum OpcodeParamMode {
    Position,
    Immediate,
}

struct OpcodeParams {
    value: i64,
}

#[derive(Clone, Copy)]
struct Opcode {
    full: i64,
}

impl Opcode {
    fn new(full_opcode: i64) -> Self {
        Opcode { full: full_opcode }
    }

    fn opcode(self) -> i64 {
        self.full % 100
    }

    fn params(self) -> OpcodeParams {
        OpcodeParams {
            value: self.full / 100,
        }
    }
}

impl Iterator for OpcodeParams {
    type Item = OpcodeParamMode;
    fn next(&mut self) -> Option<OpcodeParamMode> {
        let x = self.value % 10;
        self.value /= 10;
        match x {
            0 => Some(OpcodeParamMode::Position),
            1 => Some(OpcodeParamMode::Immediate),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Intcode {
    /// The base program, unchanging through multiple runs.
    prog: Vec<i64>,

    /// The copied program, which changes every time the program runs.
    mem: Vec<i64>,
}

#[derive(Eq, PartialEq)]
pub enum StepResult {
    Continue,
    Complete,
}

impl Intcode {
    pub fn new(prog: Vec<i64>) -> Self {
        let mut mem = Vec::new();
        mem.resize_with(prog.len(), Default::default);
        Self { prog, mem }
    }

    pub fn read<R: io::Read>(input: R) -> Result<Self> {
        Ok(Self::new(
            BufReader::new(input)
                .split(b',')
                .map(|elem| -> Result<i64> {
                    Ok(std::str::from_utf8(&elem?)?.trim_end().parse::<i64>()?)
                })
                .collect::<Result<Vec<i64>>>()?,
        ))
    }

    pub fn program(&self) -> &[i64] {
        &self.prog
    }

    pub fn memory(&self) -> &[i64] {
        &self.mem
    }

    pub fn reset_memory(&mut self) {
        self.mem.copy_from_slice(&self.prog);
    }

    pub fn run_instruction<In: FnMut() -> i64, Out: FnMut(i64)>(
        &mut self,
        pc: &mut usize,
        input: &mut In,
        output: &mut Out,
    ) -> Result<StepResult> {
        let opcode = Opcode::new(
            *self
                .mem
                .get(*pc)
                .ok_or(Error::PcOutOfBounds((*pc).try_into().unwrap()))?,
        );
        match opcode.opcode() {
            1 | 2 | 7 | 8 => {
                let mut params = opcode.params();
                let x = self.load_param(*pc + 1, params.next().unwrap())?;
                let y = self.load_param(*pc + 2, params.next().unwrap())?;
                let out_index = self.store_param(*pc + 3, params.next().unwrap())?;
                let result = match opcode.opcode() {
                    1 => x + y,
                    2 => x * y,
                    7 => (x < y) as i64,
                    8 => (x == y) as i64,
                    _ => unreachable!(),
                };
                self.set_item(out_index, result)?;
                *pc += 4;
                Ok(StepResult::Continue)
            }
            3 => {
                let out_index = self.store_param(*pc + 1, opcode.params().next().unwrap())?;
                self.set_item(out_index, input())?;
                *pc += 2;
                Ok(StepResult::Continue)
            }
            4 => {
                let value = self.load_param(*pc + 1, opcode.params().next().unwrap())?;
                output(value);
                *pc += 2;
                Ok(StepResult::Continue)
            }
            5 | 6 => {
                let mut params = opcode.params();
                let value = self.load_param(*pc + 1, params.next().unwrap())?;
                let new_pc = self
                    .load_param(*pc + 2, params.next().unwrap())?
                    .try_into()
                    .unwrap();
                let jump = match opcode.opcode() {
                    5 => value != 0,
                    6 => value == 0,
                    _ => unreachable!(),
                };
                if jump {
                    *pc = new_pc;
                } else {
                    *pc += 3;
                }
                Ok(StepResult::Continue)
            }
            99 => Ok(StepResult::Complete),
            _ => Err(Error::UnknownOpcode {
                pc: *pc,
                opcode: opcode.full,
            }),
        }
    }

    pub fn run<In: FnMut() -> i64, Out: FnMut(i64)>(
        &mut self,
        mut input: In,
        mut output: Out,
    ) -> Result<()> {
        self.reset_memory();
        let pc = &mut 0;
        while self.run_instruction(pc, &mut input, &mut output)? != StepResult::Complete {}
        Ok(())
    }

    fn get_item(&self, index: usize) -> Result<i64> {
        self.mem
            .get(index)
            .copied()
            .ok_or(Error::IndexOutOfBounds(index))
    }

    fn set_item(&mut self, index: usize, value: i64) -> Result<()> {
        *self
            .mem
            .get_mut(index)
            .ok_or(Error::IndexOutOfBounds(index))? = value;
        Ok(())
    }

    fn load_param(&self, index: usize, mode: OpcodeParamMode) -> Result<i64> {
        match mode {
            OpcodeParamMode::Position => self.get_item(self.get_item(index)?.try_into().unwrap()),
            OpcodeParamMode::Immediate => self.get_item(index),
        }
    }

    fn store_param(&self, index: usize, mode: OpcodeParamMode) -> Result<usize> {
        match mode {
            OpcodeParamMode::Position => Ok(self.get_item(index)?.try_into().unwrap()),
            OpcodeParamMode::Immediate => Err(Error::InvalidParameterMode { index }),
        }
    }
}
