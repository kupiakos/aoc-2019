use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader};

type Point = (i32, i32);
type Interval = (i32, i32);

// x1/x2 and y1/y2 go in the direction of the wire.
enum LineSegment {
    Vertical { x: i32, y1: i32, y2: i32 },
    Horizontal { x1: i32, x2: i32, y: i32 },
}

fn point_in_interval(i: &Interval, x: i32) -> bool {
    let i = normalize_interval(i);
    i.0 <= x && x <= i.1
}

fn normalize_interval(x: &Interval) -> Interval {
    if x.0 > x.1 {
        (x.1, x.0)
    } else {
        *x
    }
}

// Find some lower bound of an intersection interval, or None.
fn interval_lower_bound(a: &Interval, b: &Interval) -> Option<i32> {
    let a = normalize_interval(a);
    let b = normalize_interval(b);
    let i0 = max(a.0, b.0);
    let i1 = min(a.1, b.1);
    if i0 <= i1 {
        Some(i0)
    } else {
        None
    }
}

fn find_intersection(a: &LineSegment, b: &LineSegment) -> Option<Point> {
    match (a, b) {
        // Two vertical lines.
        (
            LineSegment::Vertical {
                x: ax,
                y1: a1,
                y2: a2,
            },
            LineSegment::Vertical {
                x: bx,
                y1: b1,
                y2: b2,
            },
        ) if ax == bx => interval_lower_bound(&(*a1, *a2), &(*b1, *b2)).map(|y| (*ax, y)),

        // Two horizontal lines.
        (
            LineSegment::Horizontal {
                y: ay,
                x1: a1,
                x2: a2,
            },
            LineSegment::Horizontal {
                y: by,
                x1: b1,
                x2: b2,
            },
        ) if ay == by => interval_lower_bound(&(*a1, *a2), &(*b1, *b2)).map(|x| (x, *ay)),

        // A vertical and horizontal line.
        (LineSegment::Horizontal { y, x1, x2 }, LineSegment::Vertical { x, y1, y2 })
        | (LineSegment::Vertical { x, y1, y2 }, LineSegment::Horizontal { y, x1, x2 })
            if point_in_interval(&(*y1, *y2), *y) && point_in_interval(&(*x1, *x2), *x) =>
        {
            Some((*x, *y))
        }
        _ => None,
    }
}

fn manhattan_distance(p: Point) -> i32 {
    p.0.abs() + p.1.abs()
}

fn parse_wire_specs(specs: &str) -> Vec<LineSegment> {
    let mut lines = Vec::new();
    let mut pos: Point = (0, 0);
    for spec in specs.split(',') {
        let dir: char = spec.chars().next().expect("direction");
        let len: i32 = spec[1..].parse().expect("length");
        let new_pos = match dir {
            'U' => (pos.0, pos.1 + len),
            'D' => (pos.0, pos.1 - len),
            'L' => (pos.0 - len, pos.1),
            'R' => (pos.0 + len, pos.1),
            d => panic!("unexpected direction: {}", d),
        };
        lines.push(match dir {
            'U' | 'D' => LineSegment::Vertical {
                x: pos.0,
                y1: pos.1,
                y2: new_pos.1,
            },
            'L' | 'R' => LineSegment::Horizontal {
                y: pos.1,
                x1: pos.0,
                x2: new_pos.0,
            },
            d => panic!("unexpected direction: {}", d),
        });
        pos = new_pos;
    }
    lines
}

fn wire_length(line: &LineSegment) -> i32 {
    match line {
        LineSegment::Horizontal { x1, x2, .. } => (x2 - x1).abs(),
        LineSegment::Vertical { y1, y2, .. } => (y2 - y1).abs(),
    }
}

fn point_on_wire_distance(line: &LineSegment, p: &Point) -> Option<i32> {
    match line {
        // Should probably check if the point is colinear but not on the line.
        LineSegment::Horizontal { x1, y, .. } if *y == p.1 => Some((p.0 - x1).abs()),
        LineSegment::Vertical { y1, x, .. } if *x == p.0 => Some((p.1 - y1).abs()),
        _ => None,
    }
}

fn find_solution(wires1: &[LineSegment], wires2: &[LineSegment]) -> (i32, i32) {
    let mut min_manhattan_intersection: i32 = std::i32::MAX;
    let mut min_wire_len_intersection: i32 = std::i32::MAX;

    let mut wire_len1: i32 = 0;
    for wire1 in wires1.iter() {
        let mut wire_len2: i32 = 0;
        for wire2 in wires2.iter() {
            match find_intersection(&wire1, &wire2) {
                Some(point) if point != (0, 0) => {
                    min_manhattan_intersection =
                        min(min_manhattan_intersection, manhattan_distance(point));
                    min_wire_len_intersection = min(
                        min_wire_len_intersection,
                        wire_len1
                            + wire_len2
                            + point_on_wire_distance(&wire1, &point).expect("wire 1 distance")
                            + point_on_wire_distance(&wire2, &point).expect("wire 2 distance"),
                    );
                }
                _ => (),
            };
            wire_len2 += wire_length(&wire2);
        }
        wire_len1 += wire_length(&wire1);
    }
    (min_manhattan_intersection, min_wire_len_intersection)
}

fn main() {
    let file = File::open("03/input.txt").expect("give me input");
    let wires: Vec<Vec<LineSegment>> = BufReader::new(file)
        .lines()
        .map(|line| parse_wire_specs(&line.unwrap()))
        .collect();
    assert_eq!(wires.len(), 2);
    println!("running...");

    let (sol1, sol2) = find_solution(&wires[0], &wires[1]);
    println!("Part 1: {}", sol1);
    println!("Part 2: {}", sol2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sols(spec1: &str, spec2: &str, manhattan: i32, wire_distance: i32) {
        let wire1 = parse_wire_specs(spec1);
        let wire2 = parse_wire_specs(spec2);
        let (sol1, sol2) = find_solution(&wire1, &wire2);
        assert_eq!(sol1, manhattan, "manhattan: {} & {}", spec1, spec2);
        assert_eq!(sol2, wire_distance, "wire_distance: {} & {}", spec1, spec2);
    }

    #[test]
    fn basic_test() {
        test_sols("R8,U5,L5,D3", "U7,R6,D4,L4", 6, 30);
        test_sols(
            "R75,D30,R83,U83,L12,D49,R71,U7,L72",
            "U62,R66,U55,R34,D71,R55,D58,R83",
            159,
            610,
        );
        test_sols(
            "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
            "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
            135,
            410,
        );
    }
}
