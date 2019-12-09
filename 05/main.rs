use std::fs::File;

mod intcode;

fn main() {
    let file = File::open("05/input.txt").expect("where input bb");
    let mut prog = intcode::read_intcode(file);
    let mut outs = Vec::new();
    intcode::run_intcode(&mut prog, || 1, |out| outs.push(out)).expect("intcode error");
    assert!(outs.len() > 2);
    assert!(outs[..outs.len() - 2].into_iter().all(|x| *x == 0));
    println!("{} tests ran.", outs.len() - 1);
    println!("Part 1: {}", outs[outs.len() - 1]);
}
