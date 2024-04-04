use std::usize;

use crate::dijkstra::Dijkstra;
use crate::dijkstra::PriorityQueue;
use crate::dimacs::*;

pub trait Lookup: Dijkstra<Vec<Vertex>> {
    fn lookup(&self, v: Vertex) -> usize;
    fn update(&mut self, v: Vertex, index: usize);
}

pub trait ImplicitHeap: Lookup {
    fn bubble_up(&mut self, dirt: usize);
    fn bubble_down(&mut self);
}

pub trait ImplicitHeapSimple: Dijkstra<Vec<Vertex>> {
    fn bubble_up(&mut self, dirt: usize);
    fn bubble_down(&mut self);
}

macro_rules! dijkstra_trait {
    ($T:ident) => {
        impl Dijkstra<Vec<Vertex>> for $T {
            fn new(n: usize, source: Vertex) -> Self {
                $T::new(n, source)
            }
            fn get_inner(&self) -> &Vec<Vertex> {
                &self.inner
            }
            fn get_mut_inner(&mut self) -> &mut Vec<Vertex> {
                &mut self.inner
            }
            fn get_dist(&self, v: Vertex) -> u32 {
                self.dist[usize::from(v)]
            }
            fn set_dist(&mut self, v: Vertex, dist: u32) {
                self.dist[usize::from(v)] = dist;
            }
            fn get_prev(&self, v: Vertex) -> Vertex {
                self.prev[usize::from(v)]
            }
            fn set_prev(&mut self, v: Vertex, prev: Vertex) {
                self.prev[usize::from(v)] = prev;
            }
            fn expanded(&self, v: Vertex) -> bool {
                self.seen[usize::from(v)]
            }
            fn mark_seen(&mut self, v: Vertex) {
                self.seen[usize::from(v)] = true;
            }
        }
    };
}

macro_rules! implicit_heap_simple {
    ($k:expr, $T:ident) => {
        pub struct $T {
            inner: Vec<Vertex>,
            dist: Vec<u32>,
            prev: Vec<Vertex>,
            seen: Vec<bool>,
        }

        impl $T {
            pub fn new(n: usize, source: Vertex) -> Self {
                let mut inner: Vec<Vertex> = Vec::new();
                let mut dist = vec![u32::MAX; n];
                inner.push(source);
                dist[usize::from(source)] = 0;
                Self {
                    inner,
                    dist,
                    prev: vec![UNDEFINED; n],
                    seen: vec![false; n],
                }
            }
        }

        dijkstra_trait!($T);

        impl ImplicitHeapSimple for $T {
            fn bubble_up(&mut self, dirt: usize) {
                let mut child = dirt;

                let mut parent;
                while child > 0 {
                    let heap = self.get_inner();
                    parent = (child - 1) / $k;
                    let parent_v = heap[parent];
                    let child_v = heap[child];
                    if self.get_dist(parent_v) <= self.get_dist(child_v) {
                        break;
                    }
                    self.get_mut_inner().swap(parent, child);
                    child = parent;
                }
            }

            fn bubble_down(&mut self) {
                let mut parent = 0;
                let n = self.get_inner().len();

                let mut child;
                while {
                    let heap = self.get_inner();
                    let base = parent * $k + 1;
                    let end = usize::min(base + $k, n);
                    child = (base..end).reduce(|left, right| {
                        if self.get_dist(heap[left]) > self.get_dist(heap[right]) {
                            right
                        } else {
                            left
                        }
                    });
                    child != None
                } {
                    let child = child.unwrap();
                    let parent_v = self.get_inner()[parent];
                    let child_v = self.get_inner()[child];
                    if self.get_dist(parent_v) <= self.get_dist(child_v) {
                        break;
                    }

                    self.get_mut_inner().swap(parent, child);
                    parent = child;
                }
            }
        }
    };
}

macro_rules! implicit_heap {
    ($k:expr, $T:ident) => {
        pub struct $T {
            inner: Vec<Vertex>,
            hlookup: Vec<usize>,
            dist: Vec<u32>,
            prev: Vec<Vertex>,
            seen: Vec<bool>,
        }

        impl $T {
            pub fn new(n: usize, source: Vertex) -> Self {
                let mut inner: Vec<Vertex> = Vec::new();
                let mut dist = vec![u32::MAX; n];
                inner.push(source);
                dist[usize::from(source)] = 0;
                Self {
                    inner,
                    hlookup: vec![0; n],
                    dist,
                    prev: vec![UNDEFINED; n],
                    seen: vec![false; n],
                }
            }
        }

        dijkstra_trait!($T);

        impl Lookup for $T {
            fn lookup(&self, v: Vertex) -> usize {
                self.hlookup[usize::from(v)]
            }
            fn update(&mut self, v: Vertex, index: usize) {
                self.hlookup[usize::from(v)] = index
            }
        }

        impl ImplicitHeap for $T {
            fn bubble_up(&mut self, dirt: usize) {
                let mut child = dirt;

                let mut parent;
                while child > 0 {
                    let heap = self.get_inner();
                    parent = (child - 1) / $k;
                    let parent_v = heap[parent];
                    let child_v = heap[child];
                    if self.get_dist(parent_v) <= self.get_dist(child_v) {
                        break;
                    }
                    self.update(parent_v, child);
                    self.update(child_v, parent);
                    self.get_mut_inner().swap(parent, child);
                    child = parent;
                }
            }

            fn bubble_down(&mut self) {
                let mut parent = 0;
                let n = self.get_inner().len();

                let mut child;
                while {
                    let heap = self.get_inner();
                    let base = parent * $k + 1;
                    let end = usize::min(base + $k, n);
                    child = (base..end).reduce(|left, right| {
                        if self.get_dist(heap[left]) > self.get_dist(heap[right]) {
                            right
                        } else {
                            left
                        }
                    });
                    child != None
                } {
                    let child = child.unwrap();
                    let parent_v = self.get_inner()[parent];
                    let child_v = self.get_inner()[child];
                    if self.get_dist(parent_v) <= self.get_dist(child_v) {
                        break;
                    }

                    self.update(parent_v, child);
                    self.update(child_v, parent);
                    self.get_mut_inner().swap(parent, child);
                    parent = child;
                }
            }
        }
    };
}

implicit_heap_simple!(2, BinaryHeapSimple);
implicit_heap_simple!(4, PentaryHeapSimple);
implicit_heap_simple!(8, OctaryHeapSimple);
implicit_heap_simple!(16, HexadecimaryHeapSimple);

implicit_heap!(2, BinaryHeap);
implicit_heap!(4, PentaryHeap);
implicit_heap!(8, OctaryHeap);
implicit_heap!(16, HexadecimaryHeap);

impl<H: ImplicitHeap> PriorityQueue for H {
    fn new(n: usize, source: Vertex) -> Self {
        H::new(n, source)
    }
    fn explore(&mut self, e: &Edge) {
        let alt = self.get_dist(e.from) + e.weight;
        if !self.expanded(e.to) && alt < self.get_dist(e.to) {
            self.set_dist(e.to, e.weight);
            if self.get_prev(e.to) == UNDEFINED {
                self.get_mut_inner().push(e.to);
                let end = self.get_inner().len() - 1;
                self.update(e.to, end);
                self.bubble_up(end);
            } else {
                self.bubble_up(self.lookup(e.to));
            }
            self.set_prev(e.to, e.from);
        }
    }

    fn pop_min(&mut self) -> Vertex {
        let heap = self.get_mut_inner();
        let last = heap
            .last()
            .expect("pop_min called even though heap was empty")
            .clone();
        self.update(last, 0);
        let min = self.get_mut_inner().swap_remove(0);
        self.bubble_down();
        self.mark_seen(min);
        return min;
    }
    fn empty(&mut self) -> bool {
        self.get_mut_inner().is_empty()
    }
}

#[cfg(test)]
mod tests {

    use crate::implicit_heaps::*;
    use rand::thread_rng;
    use rand::Rng;

    macro_rules! push_pop_test {
        // using a ty token type for macthing datatypes passed to maccro
        ($name:ident,$T:ident) => {
            #[test]
            fn $name() {
                let n = 10000;
                let mut highest_min = 0;
                let mut vertices = $T::new(n, Vertex(1));
                let mut rng = thread_rng();
                for i in 1..n {
                    let to = Vertex::try_from(i).unwrap();
                    vertices.explore(&Edge {
                        from: Vertex(1),
                        weight: rng.gen_range(1..1000000),
                        to,
                    });
                }
                //decrease_key
                for _ in 0..n {
                    let from = Vertex(1);
                    let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
                    let new = vertices.get_dist(to) / 2;
                    vertices.explore(&Edge {
                        from,
                        weight: new,
                        to,
                    });
                }
                for _ in 0..n {
                    let popped = vertices.pop_min();
                    let value = vertices.get_dist(popped);
                    assert!(value >= highest_min);
                    highest_min = u32::max(highest_min, value);
                }
                assert!(vertices.empty());
            }
        };
    }
    push_pop_test!(push_pop_2, BinaryHeap);
    push_pop_test!(push_pop_4, PentaryHeap);
    push_pop_test!(push_pop_8, OctaryHeap);
    push_pop_test!(push_pop_16, HexadecimaryHeap);
}
