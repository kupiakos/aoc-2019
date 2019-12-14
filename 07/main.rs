use intcode;
use intcode::{Intcode, Result};
use permutohedron::control::Control;
use permutohedron::heap_recursive;
use std::cmp::max;
use std::fs::File;
use std::sync::mpsc;
use std::thread;

fn test_amp(prog: &mut Intcode, sequences: &[i64]) -> Result<i64> {
    sequences.iter().copied().try_fold(0, |input, sequence| {
        let inputs = [sequence, input];
        let mut inputs_iter = inputs.iter();
        let mut output = None;
        prog.run(|| *inputs_iter.next().unwrap(), |out| output = Some(out))?;
        Ok(output.expect("no output"))
    })
}

fn test_amp_loopback(prog: &mut Intcode, sequences: &[i64]) -> Result<i64> {
    let (tx_begin, mut rx_next) = mpsc::sync_channel(0);
    // TODO: implement with a thread pool (or crazier, with async!).
    let mut threads = Vec::new();
    for (i, sequence) in sequences.into_iter().enumerate() {
        let sequence = *sequence;
        let mut prog = prog.clone();
        let mut gave_sequence = false;
        let rx = rx_next;
        let channel = mpsc::sync_channel(0);
        let tx = channel.0;
        rx_next = channel.1;
        threads.push(
            thread::Builder::new()
                .name(format!("Amp {}", i))
                .spawn(move || {
                    prog.run(
                        move || {
                            if gave_sequence {
                                rx.recv().expect("recv err")
                            } else {
                                gave_sequence = true;
                                sequence
                            }
                        },
                        move |out| tx.send(out).expect("send err"),
                    )
                    .err()
                })
                .unwrap(),
        );
    }
    let rx_end = rx_next;
    tx_begin.send(0).expect("send err at begin");
    let out = loop {
        let value = rx_end.recv().expect("recv err on end");
        // The beginning channel will disconnect when the thread finishes.
        // This notifies us that the first amp has finished and its recv end has destructed.
        // Yay for unbuffered SyncChannel!
        if let Some(err) = tx_begin.send(value).err() {
            break err.0;
        }
    };
    for t in threads {
        if let Some(err) = t.join().unwrap() {
            return Err(err);
        }
    }
    Ok(out)
}

fn test_all_amps<F: Fn(&mut Intcode, &[i64]) -> Result<i64>>(
    prog: &mut Intcode,
    sequences: &[i64],
    func: F,
) -> Result<i64> {
    let mut biggest = std::i64::MIN;
    let err = heap_recursive(&mut Vec::from(sequences), |permutation| {
        match func(prog, permutation) {
            Ok(x) => {
                biggest = max(biggest, x);
                Control::Continue
            }
            Err(err) => Control::Break(err),
        }
    })
    .break_value();
    match err {
        Some(err) => Err(err),
        None => Ok(biggest),
    }
}

fn main() {
    let file = File::open("07/input.txt").expect("where input bb");
    let prog = &mut Intcode::read(file).expect("cannot read intcode");
    let biggest_amp = match test_all_amps(prog, &[0, 1, 2, 3, 4], test_amp) {
        Ok(x) => x,
        Err(e) => panic!("intcode error, {:?}, memory:\n{:?}", e, prog.memory()),
    };
    println!("Part 1: {}", biggest_amp);
    let biggest_amp_loopback = match test_all_amps(prog, &[5, 6, 7, 8, 9], test_amp_loopback) {
        Ok(x) => x,
        Err(e) => panic!("intcode error, {:?}, memory:\n{:?}", e, prog.memory()),
    };
    println!("Part 2: {}", biggest_amp_loopback);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_max(prog_data: Vec<i64>, max_signal: i64, sequence: [i64; 5]) {
        let prog = &mut Intcode::new(prog_data);
        assert_eq!(test_amp(prog, &sequence).unwrap(), max_signal);
        assert_eq!(
            test_all_amps(prog, &sequence, test_amp).unwrap(),
            max_signal
        );
    }

    fn test_max_loopback(prog_data: Vec<i64>, max_signal: i64, sequence: [i64; 5]) {
        let prog = &mut Intcode::new(prog_data);
        assert_eq!(test_amp_loopback(prog, &sequence).unwrap(), max_signal);
        assert_eq!(
            test_all_amps(prog, &sequence, test_amp_loopback).unwrap(),
            max_signal
        );
    }

    #[test]
    fn test_maxes() {
        let prog1 = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        test_max(prog1, 43210, [4, 3, 2, 1, 0]);
        let prog2 = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        test_max(prog2, 54321, [0, 1, 2, 3, 4]);
        let prog3 = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        test_max(prog3, 65210, [1, 0, 4, 3, 2]);
    }

    #[test]
    fn test_loopback() {
        let prog1 = vec![
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ];
        test_max_loopback(prog1, 139629729, [9, 8, 7, 6, 5]);

        let prog2 = vec![
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ];
        test_max_loopback(prog2, 18216, [9, 7, 8, 5, 6]);
    }
}
