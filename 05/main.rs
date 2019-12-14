use std::fs::File;

use intcode::Intcode;

fn run_tests(prog: &mut Intcode, system_id: i64) -> i64 {
    let mut outs = Vec::new();
    prog.run(|| system_id, |out| outs.push(out))
        .expect("intcode error");
    let code = outs.pop();
    println!("{} tests ran: {:?}", outs.len(), outs);
    assert!(outs.into_iter().all(|x| x == 0));
    code.expect("no diagnostic code")
}

fn main() {
    let file = File::open("05/input.txt").expect("where input bb");
    let prog = &mut Intcode::read(file).expect("cannot read intcode");
    println!("=== Part 1 ===");
    println!("Diagnostic Code: {}", run_tests(prog, 1));
    println!("=== Part 2 ===");
    println!("Diagnostic Code: {}", run_tests(prog, 5));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_with_output(prog: &mut Intcode, input: i64, output: i64) {
        let mut out = None;
        prog.run(
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
        let prog = &mut Intcode::new(vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);

        test_with_output(prog, 7, 999);
        test_with_output(prog, 8, 1000);
        test_with_output(prog, 9, 1001);
    }
}
