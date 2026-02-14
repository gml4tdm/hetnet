///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
///////////////////////////////////////////////////////////////////////////////////////////////////

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use rand::distr::{Uniform, Distribution};
use rand::Rng;

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// Main Sampler Implementation
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AliasSampler<T> {
    _n: f64,
    probability_table: Vec<(f64, T)>,
    alias_table: Vec<T>,
    uniform: Uniform<f64>
}

impl<T: Copy> AliasSampler<T> {
    pub fn new(dist: HashMap<T, f64>) -> Self {
        let size = dist.len();
        let n = dist.len() as f64;
        let total = dist.values().sum::<f64>();
        let mut probability_table = dist.into_iter()
            .map(|(item, p)| (p * n / total, item))
            .collect::<Vec<_>>();
        let mut alias_table = Vec::with_capacity(size);

        let mut queue = WalkerInitQueue::new();
        for (i, (p, item)) in probability_table.iter().copied().enumerate() {
            queue.push(p, i);
            alias_table.push(item);
        }

        while let Some(out) = queue.pop() {
            match out {
                PopOutcome::Both((p_s, i_s), (p_l, i_l)) => {
                    let p_l_new = p_l - (1.0 - p_s);
                    probability_table[i_l].0 = p_l_new;
                    alias_table[i_s] = probability_table[i_l].1;
                    queue.push(p_l_new, i_l);
                }
                PopOutcome::Small((_p, i)) | PopOutcome::Large((_p, i)) => {
                    probability_table[i].0 = 1.0;
                }
            }
        }

        Self {
            _n: n,
            probability_table,
            alias_table,
            uniform: Uniform::new(0.0, n).expect("AliasSampler: Empty distribution")
        }
    }


    pub fn sample(&self, rng: &mut impl Rng) -> T {
        let u = self.uniform.sample(rng);
        let j = u.floor();
        let idx = j as usize;
        let (p, item) = self.probability_table[idx];
        if u - j < p {
            item
        } else {
            self.alias_table[idx]
        }
    }
}


///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// Queue Pair
///////////////////////////////////////////////////////////////////////////////////////////////////


enum PopOutcome {
    Both((f64, usize), (f64, usize)),
    Small((f64, usize)),
    Large((f64, usize)),
}


struct WalkerInitQueue {
    min_queue: MinPriorityQueue,
    max_queue: MaxPriorityQueue,
}

impl WalkerInitQueue {
    pub fn new() -> Self {
        Self {
            min_queue: MinPriorityQueue::new(), max_queue: MaxPriorityQueue::new()
        }
    }
    pub fn push(&mut self, p: f64, i: usize) {
        if p < 1.0 {
            self.min_queue.push(p, i);
        } else {
            self.max_queue.push(p, i);
        }
    }
    pub fn pop(&mut self) -> Option<PopOutcome> {
        match (self.min_queue.pop(), self.max_queue.pop()) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(PopOutcome::Both(x, y)),
            (Some(x), None) => Some(PopOutcome::Small(x)),
            (None, Some(y)) => Some(PopOutcome::Large(y)),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// Priority Queues
///////////////////////////////////////////////////////////////////////////////////////////////////

struct MaxPriorityQueue(BinaryHeap<(CmpFloat, usize)>);
struct MinPriorityQueue(BinaryHeap<(Reverse<CmpFloat>, usize)>);

impl MaxPriorityQueue {
    pub fn new() -> Self { Self(BinaryHeap::new()) }
    pub fn push(&mut self, p: f64, k: usize) { self.0.push((CmpFloat(p), k)) }
    pub fn pop(&mut self) -> Option<(f64, usize)> {
        self.0.pop().map(|(CmpFloat(x), i)| (x, i)) }
}

impl MinPriorityQueue {
    pub fn new() -> Self { Self(BinaryHeap::new()) }
    pub fn push(&mut self, p: f64, k: usize) { self.0.push((Reverse(CmpFloat(p)), k)) }
    pub fn pop(&mut self) -> Option<(f64, usize)> {
        self.0.pop().map(|(Reverse(CmpFloat(x)), i)| (x, i))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
// Comparable Float
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug, PartialOrd)]
struct CmpFloat(f64);

impl Eq for CmpFloat {}


#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for CmpFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
