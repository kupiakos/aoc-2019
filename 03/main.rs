#[macro_use]
extern crate itertools;

use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader};

// (x, y)
type Point = (i32, i32);
type Interval = (i32, i32);

// Always left to right, then bottom to top.
// type LineSegment = (Point, Point);

enum LineSegment {
    Vertical { x: i32, y1: i32, y2: i32 },
    Horizontal { x1: i32, x2: i32, y: i32 },
}
// impl Point {
//     fn cross(a: &Self, b: &Self) -> i32 {
//         a.0 * b.1 - a.1 * b.0
//     }

//     fn dot(a: &Self, b: &Self) -> i32 {
//         a.0 * b.0 + a.1 * b.1
//     }

//     fn div(a: &Self, b: i32) -> Self {
//         (a.0 / b, a.1 / b)
//     }
// }

// fn find_line_intersection(a: &LineSegment, b: &LineSegment) -> Option<Point> {
//     // https://stackoverflow.com/a/565282
//     let p = a.0;
//     let q = b.0;
//     let r = a.1 - a.0;
//     let s = b.1 - b.0;

//     let r_cross_s = Point::cross(&r, &s);
//     let q_minus_p = q - p;
//     let q_minus_p_cross_r = Point::cross(&q_minus_p, &r);

//     // Are the lines parallel?
//     if r_cross_s == 0 {
//         if q_minus_p_cross_r == 0 {
//             // They are collinear. Do they overlap?
//             let r_dot_r = Point::dot(&r, &r);
//             let t0 = Point::dot(&q_minus_p, &r) / r_dot_r;
//             let t1 = t0 + Point::dot(&s, &r) / r_dot_r;
//             if t0 > t1 {
//                 std::mem::swap(&mut t0, &mut t1)
//             }
//             t0 <= 1 && 0 <= t1
//         } else {
//             // They are parallel but not collinear.
//             false
//         }
//     } else {
//         let t =
//     }
// }

fn point_in_interval(i: &Interval, x: i32) -> bool {
    i.0 <= x && x <= i.1
}

// Find some lower bound of an intersection interval, or None.
fn interval_lower_bound(a: &Interval, b: &Interval) -> Option<i32> {
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

fn find_all_intersections<'a>(
    set1: &'a [LineSegment],
    set2: &'a [LineSegment],
) -> impl Iterator<Item = Point> + 'a {
    // Premature optimization is the root of all evil.
    // This implements the O(N^2) algorithm, although I could get this down to O(N log N).
    iproduct!(set1.iter(), set2.iter()).filter_map(|(a, b)| find_intersection(a, b))
}

fn manhattan_distance_nonzero(p: Point) -> Option<i32> {
    Some(p.0.abs() + p.1.abs()).filter(|x| *x != 0)
}

fn find_closest_intersection(set1: &[LineSegment], set2: &[LineSegment]) -> Option<i32> {
    find_all_intersections(set1, set2).filter_map(manhattan_distance_nonzero).min()
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
            'U' => LineSegment::Vertical {
                x: pos.0,
                y1: pos.1,
                y2: new_pos.1,
            },
            'D' => LineSegment::Vertical {
                x: pos.0,
                y1: new_pos.1,
                y2: pos.1,
            },
            'L' => LineSegment::Horizontal {
                y: pos.1,
                x1: new_pos.0,
                x2: pos.0,
            },
            'R' => LineSegment::Horizontal {
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

fn main() {
    let file = File::open("03/input.txt").expect("give me input");
    let wires: Vec<Vec<LineSegment>> = BufReader::new(file)
        .lines()
        .map(|line| parse_wire_specs(&line.unwrap()))
        .collect();
    assert_eq!(wires.len(), 2);
    let closest_intersection = find_closest_intersection(&wires[0], &wires[1]).expect("no intersection");
    println!("running...");
    println!("Part 1: {}", closest_intersection);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cross(spec1: &str, spec2: &str, expected_distance: i32) {
        let wire1 = parse_wire_specs(spec1);
        let wire2 = parse_wire_specs(spec2);
        assert_eq!(
            find_closest_intersection(&wire1, &wire2).expect("no intersection"), expected_distance);
    }

    #[test]
    fn basic_test() {
        test_cross("R8,U5,L5,D3", "U7,R6,D4,L4", 6);
        test_cross("R75,D30,R83,U83,L12,D49,R71,U7,L72",
            "U62,R66,U55,R34,D71,R55,D58,R83", 159);
        test_cross("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
            "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7", 135);
    }
}
