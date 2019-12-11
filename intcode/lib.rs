use std::convert::TryInto;
use std::io::Read;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum Error {
    IndexOutOfBounds(i64),
    PcOutOfBounds(i64),
    UnknownOpcode(i64),
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

fn get_item(prog: &[i64], index: usize) -> Result<i64> {
    prog.get(index)
        .copied()
        .ok_or(Error::IndexOutOfBounds(index.try_into().unwrap()))
}

fn set_item(prog: &mut [i64], index: usize, value: i64) -> Result<()> {
    *prog
        .get_mut(index)
        .ok_or(Error::IndexOutOfBounds(index.try_into().unwrap()))? = value;
    Ok(())
}

fn load_param(prog: &[i64], index: usize, mode: OpcodeParamMode) -> Result<i64> {
    match mode {
        OpcodeParamMode::Position => get_item(prog, get_item(prog, index)?.try_into().unwrap()),
        OpcodeParamMode::Immediate => get_item(prog, index),
    }
}

fn store_param(prog: &[i64], index: usize, mode: OpcodeParamMode) -> Result<usize> {
    match mode {
        OpcodeParamMode::Position => Ok(get_item(prog, index)?.try_into().unwrap()),
        OpcodeParamMode::Immediate => panic!("cannot have immediate param mode for store param"),
    }
}

pub fn read_intcode<R: Read>(input: R) -> Vec<i64> {
    BufReader::new(input)
        .split(b',')
        .map(|elem| {
            std::str::from_utf8(&elem.unwrap())
                .unwrap()
                .trim_end()
                .parse()
                .expect("not an int?")
        })
        .collect()
}

pub fn run_intcode<In: FnMut() -> i64, Out: FnMut(i64)>(
    prog: &mut [i64],
    mut input: In,
    mut output: Out,
) -> Result<()> {
    let mut pc: usize = 0;
    loop {
        let opcode = Opcode::new(
            *prog
                .get(pc)
                .ok_or(Error::PcOutOfBounds(pc.try_into().unwrap()))?,
        );
        match opcode.opcode() {
            1 | 2 | 7 | 8 => {
                let mut params = opcode.params();
                let x = load_param(prog, pc + 1, params.next().unwrap())?;
                let y = load_param(prog, pc + 2, params.next().unwrap())?;
                let out_index = store_param(prog, pc + 3, params.next().unwrap())?;
                let result = match opcode.opcode() {
                    1 => x + y,
                    2 => x * y,
                    7 => (x < y) as i64,
                    8 => (x == y) as i64,
                    _ => unreachable!(),
                };
                set_item(prog, out_index, result)?;
                pc += 4;
            }
            3 => {
                let out_index = store_param(prog, pc + 1, opcode.params().next().unwrap())?;
                set_item(prog, out_index, input())?;
                pc += 2
            }
            4 => {
                let value = load_param(prog, pc + 1, opcode.params().next().unwrap())?;
                output(value);
                pc += 2;
            }
            5 | 6 => {
                let mut params = opcode.params();
                let value = load_param(prog, pc + 1, params.next().unwrap())?;
                let new_pc = load_param(prog, pc + 2, params.next().unwrap())?
                    .try_into()
                    .unwrap();
                let jump = match opcode.opcode() {
                    5 => value != 0,
                    6 => value == 0,
                    _ => unreachable!(),
                };
                if jump {
                    pc = new_pc;
                } else {
                    pc += 3;
                }
            }
            99 => return Ok(()),
            _ => return Err(Error::UnknownOpcode(opcode.full)),
        };
    }
}
