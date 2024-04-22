use std::slice::Iter;
use std::usize;

use crate::dimacs::*;

pub struct SimpleList {
    inner: Vec<Vertex>,
    dist: Vec<u32>,
    prev: Vec<Vertex>,
    seen: Vec<bool>,
}

impl SimpleList {
    pub fn new(n: usize, source: Vertex) -> Self {
        let mut inner: Vec<Vertex> = Vec::new();
        let mut dist: Vec<u32> = Vec::with_capacity(n);
        let mut prev: Vec<Vertex> = Vec::with_capacity(n);
        let mut seen: Vec<bool> = Vec::with_capacity(n);

        inner.push(source);

        for i in 0..n {
            let v = Vertex::try_from(i).unwrap();
            dist.push(if v == source { 0 } else { u32::MAX });
            prev.push(UNDEFINED);
            seen.push(false);
        }
        Self {
            inner,
            dist,
            prev,
            seen,
        }
    }
}

pub trait Dijkstra {
    type Inner;

    fn new(n: usize, source: Vertex) -> Self;
    fn get_inner(&self) -> &Self::Inner;
    fn get_mut_inner(&mut self) -> &mut Self::Inner;
    fn get_dist(&self, v: Vertex) -> u32;
    fn set_dist(&mut self, v: Vertex, dist: u32);
    fn get_prev(&self, v: Vertex) -> Vertex;
    fn set_prev(&mut self, v: Vertex, prev: Vertex);
    fn expanded(&self, v: Vertex) -> bool;
    fn mark_seen(&mut self, v: Vertex);
}

pub trait PriorityQueue<I, E> {
    fn new(n: usize, source: Vertex) -> Self;
    fn explore(&mut self, from: I, e: &E);
    fn pop_min(&mut self) -> I;
    fn empty(&mut self) -> bool;
}

impl Dijkstra for SimpleList {

    type Inner = Vec<Vertex>;
    
    fn new(n: usize, source: Vertex) -> Self {
        SimpleList::new(n, source)
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

impl PriorityQueue<Vertex, Neighbor> for SimpleList {
    fn new(n: usize, source: Vertex) -> Self {
        SimpleList::new(n, source)
    }
    fn pop_min(&mut self) -> Vertex {
        let list = self.get_inner();
        let i = list
            .iter()
            .enumerate()
            .min_by(|(_, &a), (_, &b)| self.get_dist(a).cmp(&self.get_dist(b)))
            .map(|(index, _)| index)
            .unwrap();
        let min = self.get_mut_inner().swap_remove(i);
        self.mark_seen(min);
        return min;
    }
    fn explore(&mut self, from: Vertex, e: &Neighbor) {
        let alt = self.get_dist(from) + e.weight;
        if !self.expanded(e.to) && alt < self.get_dist(e.to) {
            if self.get_prev(e.to) == UNDEFINED {
                self.get_mut_inner().push(e.to);
            }
            self.set_dist(e.to, alt);
            self.set_prev(e.to, from);
        }
    }
    fn empty(&mut self) -> bool {
        self.get_mut_inner().is_empty()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Neighbor {
    pub to: Vertex,
    pub weight: u32,
}

impl From<Edge> for Neighbor {
    fn from(value: Edge) -> Self {
        Neighbor {
            to: value.to,
            weight: value.weight,
        }
    }
}

pub type NeighborList = Vec<Vec<Neighbor>>;

pub trait StructuredEdges<E> {
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self;
    fn get_neighbors(&self, u: Vertex) -> Iter<E>;
}

impl StructuredEdges<Neighbor> for NeighborList {
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self {
        let mut out_edges: Vec<Vec<Neighbor>> = vec![Vec::new(); n];

        for e in edges {
            out_edges[usize::from(e.from)].push(Neighbor::from(e));
        }
        return out_edges;
    }
    fn get_neighbors(&self, u: Vertex) -> Iter<Neighbor> {
        self[usize::from(u)].iter()
    }
}

pub fn dijkstra<Q, S, I, E>(mut queue: Q, edges: &S) -> Q
where
    Q: PriorityQueue<I, E>,
    S: StructuredEdges<E>,
    I: Copy,
    Vertex: From<I>,
{
    while !queue.empty() {
        //choose next vector
        let u = queue.pop_min();

        // update neighbors of u
        for e in edges.get_neighbors(Vertex::from(u)) {
            queue.explore(u, &e);
        }
    }
    return queue;
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn push_pop_list() {
        let n = 10000;
        let mut highest_min = 0;
        let mut vertices = SimpleList::new(n, Vertex(1));
        let mut rng = thread_rng();
        //push
        for i in 1..n {
            let to = Vertex::try_from(i).unwrap();
            vertices.explore(
                Vertex(1),
                &Neighbor {
                    weight: rng.gen_range(1..1000000),
                    to,
                },
            );
        }
        //decrease_key
        for _ in 0..n {
            let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let new = vertices.get_dist(to) / 2;
            vertices.explore(Vertex(1), &Neighbor { weight: new, to });
        }
        //pop
        for _ in 0..n {
            let popped = vertices.pop_min();
            let value = vertices.get_dist(popped);
            assert!(value >= highest_min);
            highest_min = u32::max(highest_min, value);
        }
        assert!(vertices.empty());
    }
}
