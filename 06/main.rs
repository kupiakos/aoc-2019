use indextree::{Arena, NodeId};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct OrbitTree {
    arena: Arena<String>,
    ids: HashMap<String, NodeId>,
}

impl OrbitTree {
    pub fn new() -> Self {
        OrbitTree {
            arena: Arena::new(),
            ids: HashMap::new(),
        }
    }

    fn find_com(&self) -> Option<NodeId> {
        self.ids.values().next()?.ancestors(&self.arena).last()
    }

    pub fn add_orbit(&mut self, src: &str, dst: &str) {
        let arena = &mut self.arena;
        let ids = &mut self.ids;
        let src_id = *ids
            .entry(src.to_string())
            .or_insert_with(|| arena.new_node(src.to_string()));
        let dst_id = *ids
            .entry(dst.to_string())
            .or_insert_with(|| arena.new_node(dst.to_string()));
        src_id.append(dst_id, arena);
    }

    pub fn orbit_count_checksum(&self) -> u32 {
        self.count_orbits(self.find_com().expect("no com?"), 0)
    }

    fn count_orbits(&self, node: NodeId, level: u32) -> u32 {
        level
            + node
                .children(&self.arena)
                .map(|child| self.count_orbits(child, level + 1))
                .sum::<u32>()
    }

    pub fn orbit_distance(&self, src: &str, dst: &str) -> Option<usize> {
        // O(N^2) solution to find the indices of the first common element in two arrays.
        let dst_ancestors: Vec<&str> = self
            .ids
            .get(dst)?
            .ancestors(&self.arena)
            .skip(1)
            .filter_map(|x| self.arena.get(x).map(|y| y.get().as_str()))
            .collect();
        self.ids
            .get(src)?
            .ancestors(&self.arena)
            .skip(1)
            .enumerate()
            .filter_map(|(i, x)| {
                let x = self.arena.get(x)?.get().as_str();
                dst_ancestors.iter().position(|&y| x == y).map(|j| i + j)
            })
            .nth(0)
    }
}

fn main() {
    let file = File::open("06/input.txt").expect("give me input or i will scream");
    let mut tree = OrbitTree::new();
    for line in BufReader::new(file).lines() {
        if let &[src, dst] = line.unwrap().split(')').collect::<Vec<_>>().as_slice() {
            tree.add_orbit(src, dst);
        }
    }
    println!("Part 1: {}", tree.orbit_count_checksum());
    println!(
        "Part 2: {}",
        tree.orbit_distance("YOU", "SAN")
            .expect("no orbit distance?")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_orbit() {
        let orbits = [
            ("COM", "B"),
            ("B", "C"),
            ("C", "D"),
            ("D", "E"),
            ("E", "F"),
            ("B", "G"),
            ("G", "H"),
            ("D", "I"),
            ("E", "J"),
            ("J", "K"),
            ("K", "L"),
        ];
        let mut tree = OrbitTree::new();
        for (src, dst) in orbits.into_iter() {
            tree.add_orbit(src, dst);
        }
        assert_eq!(tree.orbit_count_checksum(), 42);
    }
}
