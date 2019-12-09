use std::fs::File;

mod intcode;

fn run_tests(prog: &mut [i64], system_id: i64) -> i64 {
    let mut outs = Vec::new();
    intcode::run_intcode(prog, || system_id, |out| outs.push(out)).expect("intcode error");
    let code = outs.pop();
    println!("{} tests ran: {:?}", outs.len(), outs);
    assert!(outs.into_iter().all(|x| x == 0));
    code.expect("no diagnostic code")
}

fn main() {
    let file = File::open("05/input.txt").expect("where input bb");
    let prog = intcode::read_intcode(file);
    println!("=== Part 1 ===");
    println!("Diagnostic Code: {}", run_tests(&mut prog.clone(), 1));
    println!("=== Part 2 ===");
    println!("Diagnostic Code: {}", run_tests(&mut prog.clone(), 5));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_with_output(prog: &mut [i64], input: i64, output: i64) {
        let mut out = None;
        intcode::run_intcode(
            prog,
            || input,
            |x| {
                assert!(out.is_none());
                out = Some(x);
            },
        )
        .expect("intcode error");
        assert_eq!(out.expect("no output"), output);
    }

    #[test]
    fn large_example() {
        let prog = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        test_with_output(&mut prog.clone(), 7, 999);
        test_with_output(&mut prog.clone(), 8, 1000);
        test_with_output(&mut prog.clone(), 9, 1001);
    }
}
