use alloc::vec::Vec;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Node {
    pub numbers: Vec<usize>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(mut n: usize, rng: &mut impl Rng) -> Node {
        let mut children = Vec::new();
        let mut numbers = Vec::new();

        match n < 2 {
            true => {
                for _ in 0..n {
                    numbers.push(0);
                    children.push(Node::new(0, rng));
                }
            }
            false => {
                while n > 0 {
                    let n2 = rng.random_range(0..=n);
                    numbers.push(n2);
                    children.push(Node::new(n2, rng));
                    n -= n2;
                }
            }
        }

        Self { numbers, children }
    }

    fn num_children(&self) -> usize {
        let children = self.children.len();
        let grand_children: usize = self.children.iter().map(|x| x.num_children()).sum();
        children + grand_children
    }
}

pub struct Roots(pub Vec<Node>);

impl Roots {
    pub fn new(num_roots: usize, n: usize, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        Self((0..num_roots).map(|_| Node::new(n, &mut rng)).collect())
    }

    pub fn num_nodes(&self) -> usize {
        self.0.len() + self.0.iter().map(|x| x.num_children()).sum::<usize>()
    }

    pub fn as_slice(&self) -> &[Node] {
        &self.0
    }
}
