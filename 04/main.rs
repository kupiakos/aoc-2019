#![feature(is_sorted)]

use std::ops::Range;

/// Iterates adjacent equal groups of elements and their counts.
/// For example, [1, 2, 2, 2, 3, 9, 9] yields [(1, 1), (2, 3), (3, 1), (9, 2)].
struct Groups<I>
where
    I: Iterator,
{
    last: Option<I::Item>,
    last_cnt: usize,
    base: I,
}

impl<I> Iterator for Groups<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    type Item = (I::Item, usize);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(this) = self.base.next() {
            match self.last.take() {
                Some(last) if last != this => {
                    let last_cnt = self.last_cnt;
                    self.last = Some(this);
                    self.last_cnt = 1;
                    return Some((last, last_cnt));
                }
                _ => {
                    self.last = Some(this);
                    self.last_cnt += 1;
                }
            }
        }
        self.last.take().map(|l| (l, self.last_cnt))
    }
}

trait GroupsExt: Iterator {
    fn groups(self) -> Groups<Self>
    where
        Self::Item: PartialEq,
        Self: Sized,
    {
        Groups {
            last: None,
            last_cnt: 0,
            base: self,
        }
    }
}
impl<I: Iterator> GroupsExt for I {}

// Brute-forced method.
fn is_valid_password(pwd: i32) -> bool {
    let pwd_str = pwd.to_string().into_bytes();
    pwd > 0
        && pwd_str.len() == 6
        && pwd_str.iter().groups().filter(|(_x, n)| *n >= 2).count() >= 1
        && pwd_str.iter().is_sorted()
}

fn is_valid_password_revised(pwd: i32) -> bool {
    let pwd_str = pwd.to_string().into_bytes();

    pwd > 0
        && pwd_str.len() == 6
        && pwd_str.iter().groups().filter(|(_x, n)| *n == 2).count() >= 1
        && pwd_str.iter().is_sorted()
}

fn main() {
    println!("solving...");
    const RANGE: Range<i32> = 147981..691423;
    let sol1 = RANGE.filter(|pwd| is_valid_password(*pwd)).count();
    let sol2 = RANGE.filter(|pwd| is_valid_password_revised(*pwd)).count();
    println!("Part 1: {}", sol1);
    println!("Part 2: {}", sol2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_passwords() {
        assert!(is_valid_password(111111));
        assert!(is_valid_password(111123));
        assert!(is_valid_password(135579));
        assert!(is_valid_password(122345));
    }

    #[test]
    fn invalid_passwords() {
        assert!(!is_valid_password(223450));
        assert!(!is_valid_password(123789));
        assert!(!is_valid_password(-1));
        assert!(!is_valid_password(0));
        assert!(!is_valid_password(100000));
    }

    #[test]
    fn valid_passwords_revised() {
        assert!(is_valid_password_revised(111122));
        assert!(is_valid_password_revised(135579));
        assert!(is_valid_password_revised(122345));
    }

    #[test]
    fn invalid_passwords_revised() {
        assert!(!is_valid_password_revised(111111));
        assert!(!is_valid_password_revised(123444));
        assert!(!is_valid_password_revised(223450));
        assert!(!is_valid_password_revised(123789));
        assert!(!is_valid_password_revised(-1));
        assert!(!is_valid_password_revised(0));
        assert!(!is_valid_password_revised(100000));
    }
}
