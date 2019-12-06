use std::cmp::max;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn fuel_for_mass_trivial(mass: i64) -> i64 {
    max(0, (mass / 3) - 2)
}

fn fuel_for_mass(mass: i64) -> i64 {
    let mut sum = 0;
    let mut stage_mass = mass;
    loop {
        stage_mass = fuel_for_mass_trivial(stage_mass);
        if stage_mass == 0 {
            break;
        }
        sum += stage_mass;
    }
    sum
}

fn main() {
    let file = File::open("01/input.txt").expect("give me input");
    let masses: Vec<i64> = BufReader::new(file)
        .lines()
        .map(|line| {
            line.expect("line error")
                .parse::<i64>()
                .expect("parse error")
        })
        .collect();
    println!(
        "Part 1: {}",
        masses
            .iter()
            .copied()
            .map(fuel_for_mass_trivial)
            .sum::<i64>()
    );
    println!(
        "Part 2: {}",
        masses.iter().copied().map(fuel_for_mass).sum::<i64>()
    );
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn trivial_mass() {
        assert_eq!(fuel_for_mass_trivial(12), 2);
        assert_eq!(fuel_for_mass_trivial(14), 2);
        assert_eq!(fuel_for_mass_trivial(1969), 654);
        assert_eq!(fuel_for_mass_trivial(100756), 33583);
    }

    #[test]
    fn full_mass() {
        assert_eq!(fuel_for_mass(14), 2);
        assert_eq!(fuel_for_mass(1969), 966);
        assert_eq!(fuel_for_mass(100756), 50346);
    }
}
