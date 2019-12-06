use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, Mul};

#[derive(Debug)]
enum Error {
    IndexOutOfBounds(i64),
    PcOutOfBounds(i64),
    UnknownOpcode(i64),
}

fn get_item(prog: &[i64], index: usize) -> Result<i64, Error> {
    prog.get(index)
        .copied()
        .ok_or(Error::IndexOutOfBounds(index.try_into().unwrap()))
}

fn set_item(prog: &mut [i64], index: usize, value: i64) -> Result<(), Error> {
    *prog
        .get_mut(index)
        .ok_or(Error::IndexOutOfBounds(index.try_into().unwrap()))? = value;
    Ok(())
}

fn run_intcode(prog: &mut [i64]) -> Result<(), Error> {
    let mut pc: usize = 0;
    loop {
        let opcode = *prog
            .get(pc)
            .ok_or(Error::PcOutOfBounds(pc.try_into().unwrap()))?;
        match opcode {
            1 | 2 => {
                let i1 = get_item(prog, pc + 1)?.try_into().unwrap();
                let i2 = get_item(prog, pc + 2)?.try_into().unwrap();
                let out_index = get_item(prog, pc + 3)?.try_into().unwrap();
                let op: fn(i64, i64) -> i64 = if opcode == 1 { i64::add } else { i64::mul };
                let result = op(get_item(prog, i1)?, get_item(prog, i2)?);
                set_item(prog, out_index, result)?;
            }
            99 => return Ok(()),
            _ => return Err(Error::UnknownOpcode(opcode)),
        };
        pc += 4;
    }
}

fn run_gravity_program(prog: &[i64], noun: i64, verb: i64) -> Result<i64, Error> {
    let mut prog_copy = Vec::from(prog);
    prog_copy[1] = noun;
    prog_copy[2] = verb;
    run_intcode(&mut prog_copy)?;
    Ok(prog_copy[0])
}

fn main() {
    let file = File::open("02/input.txt").expect("give me input");
    let prog: Vec<i64> = BufReader::new(file)
        .split(b',')
        .map(|elem| {
            std::str::from_utf8(&elem.unwrap())
                .unwrap()
                .trim_end()
                .parse()
                .expect("not an int?")
        })
        .collect();
    println!(
        "Part 1: {}",
        run_gravity_program(&prog, 12, 2).expect("run intcode")
    );
    println!("Bruteforcing solution for part 2...");
    const DESIRED_SOLUTION: i64 = 19690720;
    for noun in 0..=99 {
        for verb in 0..=99 {
            if run_gravity_program(&prog, noun, verb).expect("run intcode") == DESIRED_SOLUTION {
                println!("Part 2: {} (100 * {} + {})", 100 * noun + verb, noun, verb);
                return;
            }
        }
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    fn test_example(input: &[i64], expected_output: &[i64]) {
        let mut input = Vec::from(input);
        run_intcode(&mut input).expect("run intcode");
        assert_eq!(input, expected_output);
    }

    #[test]
    fn given_examples() {
        test_example(
            &[1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            &[3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50],
        );
        test_example(&[1, 0, 0, 0, 99], &[2, 0, 0, 0, 99]);
        test_example(&[2, 3, 0, 3, 99], &[2, 3, 0, 6, 99]);
        test_example(&[2, 4, 4, 5, 99, 0], &[2, 4, 4, 5, 99, 9801]);
        test_example(
            &[1, 1, 1, 4, 99, 5, 6, 0, 99],
            &[30, 1, 1, 4, 2, 5, 6, 0, 99],
        );
    }
}
