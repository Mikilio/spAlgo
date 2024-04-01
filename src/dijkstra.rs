use std::usize;

use crate::dimacs::*;

pub struct SimpleList {
    queue: Vec<Vertex>,
    pub dist: Vec<u32>,
    pub prev: Vec<Vertex>,
    seen: Vec<bool>,
}

pub struct BinaryHeap {
    heap: Vec<Vertex>,
    hlookup: Vec<usize>,
    pub dist: Vec<u32>,
    pub prev: Vec<Vertex>,
    seen: Vec<bool>,
}

impl BinaryHeap {
    fn bubble_up(&mut self, dirt: usize) {
        let mut child = dirt;
        let heap = &mut (self.heap);

        let mut parent;
        while child > 0 {
            parent = (child - 1) / 2;
            let parent_v_i = usize::from(heap[parent]);
            let child_v_i = usize::from(heap[child]);
            if self.dist[parent_v_i] <= self.dist[child_v_i] {
                break;
            }
            self.hlookup[parent_v_i] = child;
            self.hlookup[child_v_i] = parent;
            heap.swap(parent, child);
            child = parent;
        }
    }

    fn bubble_down(&mut self) {
        let mut parent = 0;
        let n = self.heap.len();
        let heap = &mut (self.heap);

        let mut child;
        while {
            let left = parent * 2 + 1;
            let right = left + 1;
            child = if right < n
                && self.dist[usize::from(heap[left])] > self.dist[usize::from(heap[right])]
            {
                right
            } else {
                left
            };
            child < n
        } {
            let parent_v_i = usize::from(heap[parent]);
            let child_v_i = usize::from(heap[child]);
            if self.dist[parent_v_i] <= self.dist[child_v_i] {
                break;
            }
            self.hlookup[parent_v_i] = child;
            self.hlookup[child_v_i] = parent;
            heap.swap(parent, child);
            parent = child;
        }
    }
}

pub trait PriorityQueue {
    fn new(n: usize, source: Vertex) -> Self;
    fn update_vertice(&mut self, v: Vertex, dist: u32, prev: Vertex);
    fn get_dist(&mut self, v: Vertex) -> u32;
    fn expanded(&mut self, v: Vertex) -> bool;
    fn pop_min(&mut self) -> Vertex;
    fn empty(&mut self) -> bool;
}

impl PriorityQueue for SimpleList {
    fn new(n: usize, source: Vertex) -> Self {
        let mut queue: Vec<Vertex> = Vec::new();
        let mut dist: Vec<u32> = Vec::with_capacity(n);
        let mut prev: Vec<Vertex> = Vec::with_capacity(n);
        let mut seen: Vec<bool> = Vec::with_capacity(n);

        queue.push(source);

        for i in 0..n {
            let v = Vertex::try_from(i).unwrap();
            dist.push(if v == source { 0 } else { u32::MAX });
            prev.push(UNDEFINED);
            seen.push(false);
        }
        Self {
            queue,
            dist,
            prev,
            seen,
        }
    }
    fn update_vertice(&mut self, v: Vertex, dist: u32, prev: Vertex) {
        if self.prev[usize::from(v)] == UNDEFINED {
            self.queue.push(v);
        }
        self.dist[usize::from(v)] = dist;
        self.prev[usize::from(v)] = prev;
    }
    fn get_dist(&mut self, v: Vertex) -> u32 {
        self.dist[usize::from(v)]
    }
    fn expanded(&mut self, v: Vertex) -> bool {
        self.seen[usize::from(v)]
    }
    fn pop_min(&mut self) -> Vertex {
        let i = self
            .queue
            .iter()
            .enumerate()
            .min_by(|(_, &a), (_, &b)| {
                (&(self.dist)[usize::from(a)]).cmp(&(self.dist)[usize::from(b)])
            })
            .map(|(index, _)| index)
            .unwrap();
        let min = self.queue.swap_remove(i);
        self.seen[usize::from(min)] = true;
        return min;
    }
    fn empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

impl PriorityQueue for BinaryHeap {
    fn new(n: usize, source: Vertex) -> Self {
        let list = SimpleList::new(n, source);
        let mut hlookup = vec![usize::MAX; n];
        hlookup[usize::from(source)] = 0;
        Self {
            heap: list.queue,
            hlookup,
            dist: list.dist,
            prev: list.prev,
            seen: list.seen,
        }
    }

    fn update_vertice(&mut self, v: Vertex, dist: u32, prev: Vertex) {
        self.dist[usize::from(v)] = dist;
        if self.prev[usize::from(v)] == UNDEFINED {
            self.heap.push(v);
            let end = self.heap.len() - 1;
            self.hlookup[usize::from(v)] = end;
            self.bubble_up(end);
        } else {
            self.bubble_up(self.hlookup[usize::from(v)]);
        }
        self.prev[usize::from(v)] = prev;
    }
    fn get_dist(&mut self, v: Vertex) -> u32 {
        self.dist[usize::from(v)]
    }
    fn expanded(&mut self, v: Vertex) -> bool {
        self.seen[usize::from(v)]
    }
    fn pop_min(&mut self) -> Vertex {
        let last = self
            .heap
            .last()
            .expect("pop_min called even though heap was empty")
            .clone();
        self.hlookup[usize::from(last)] = 0;
        let min = self.heap.swap_remove(0);
        self.bubble_down();
        self.seen[usize::from(min)] = true;
        return min;
    }
    fn empty(&mut self) -> bool {
        self.heap.is_empty()
    }
}

pub struct NeighborList(Vec<Vec<Edge>>);

pub trait StructuredEdges {
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self;
    fn get_neighbors(&self, u: Vertex) -> impl Iterator<Item = &Edge>;
}

impl StructuredEdges for NeighborList {
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self {
        let mut out_edges: Vec<Vec<Edge>> = vec![Vec::new(); n];

        for e in edges {
            out_edges[usize::from(e.from)].push(e);
        }
        return NeighborList(out_edges);
    }
    fn get_neighbors(&self, u: Vertex) -> impl Iterator<Item = &Edge> {
        self.0[usize::from(u)].iter()
    }
}

pub fn dijkstra<Q, E>(mut queue: Q, edges: &E) -> Q
where
    Q: PriorityQueue,
    E: StructuredEdges,
{
    while !queue.empty() {
        //choose next vector
        let u = queue.pop_min();

        // update neighbors of u
        for e in edges.get_neighbors(u) {
            let alt = queue.get_dist(e.from) + e.weight;
            if !queue.expanded(e.to) && alt < queue.get_dist(e.to) {
                queue.update_vertice(e.to, alt, u);
            }
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
            let new_vertex = Vertex::try_from(i).unwrap();
            vertices.update_vertice(new_vertex, rng.gen_range(1..1000000), Vertex(1))
        }
        //decrease_key
        for _ in 0..n {
            let v: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let new = vertices.get_dist(v) / 2;
            vertices.update_vertice(v, new, to);
        }
        //pop
        for _ in 0..n {
            let popped = vertices.pop_min();
            let value = vertices.dist[usize::from(popped)];
            assert!(value >= highest_min);
            highest_min = u32::max(highest_min, value);
        }
        assert!(vertices.empty());
    }
    #[test]
    fn push_pop_heap() {
        let n = 10000;
        let mut highest_min = 0;
        let mut vertices = BinaryHeap::new(n, Vertex(1));
        let mut rng = thread_rng();
        for i in 1..n {
            let new_vertex = Vertex::try_from(i).unwrap();
            vertices.update_vertice(new_vertex, rng.gen_range(1..1000000), Vertex(1))
        }
        //decrease_key
        for _ in 0..n {
            let v: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let new = vertices.get_dist(v) / 2;
            vertices.update_vertice(v, new, to);
        }
        for _ in 0..n {
            let popped = vertices.pop_min();
            let value = vertices.dist[usize::from(popped)];
            assert!(value >= highest_min);
            highest_min = u32::max(highest_min, value);
        }
        assert!(vertices.empty());
    }
}
