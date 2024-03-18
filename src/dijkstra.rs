use crate::dimacs::*;
use std::io::prelude::*;

pub struct VertexList {
    queue: Vec<Vertex>,
    pub dist: Vec<u32>,
    pub prev: Vec<Vertex>,
    seen: Vec<bool>,
}

impl VertexList {
    pub fn new(n: usize, source: Vertex) -> Self {
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
}

pub struct VertexHeap {
    queue: Vec<Vertex>,
    pub dist: Vec<u32>,
    pub prev: Vec<Vertex>,
    seen: Vec<bool>,
}

impl VertexHeap {
    pub fn new(n: usize, source: Vertex) -> Self {
        let list = VertexList::new(n, source);
        Self {
            queue: list.queue,
            dist: list.dist,
            prev: list.prev,
            seen: list.seen,
        }
    }

    fn bubble_up(&mut self) {
        let mut child = self.queue.len() - 1;
        let heap = &mut (self.queue);

        let mut parent;
        while child > 0 {
            parent = (child - 1) / 2;
            if self.dist[usize::from(heap[parent])] <= self.dist[usize::from(heap[child])] {
                break;
            }
            heap.swap(parent, child);
            child = parent;
        }
    }

    fn bubble_down(&mut self) {
        let mut parent = 0;
        let n = self.queue.len();
        let heap = &mut (self.queue);

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
            if self.dist[usize::from(heap[parent])] <= self.dist[usize::from(heap[child])] {
                break;
            }
            heap.swap(parent, child);
            parent = child;
        }
    }
}

pub trait PriorityQueue {
    fn update_vertice(&mut self, v: Vertex, dist: u32, prev: Vertex);
    fn get_dist(&mut self, v: Vertex) -> u32;
    fn expanded(&mut self, v: Vertex) -> bool;
    fn pop_min(&mut self) -> Vertex;
    fn empty(&mut self) -> bool;
}

impl PriorityQueue for VertexList {
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

impl PriorityQueue for VertexHeap {
    fn update_vertice(&mut self, v: Vertex, dist: u32, prev: Vertex) {
        self.dist[usize::from(v)] = dist;
        if self.prev[usize::from(v)] == UNDEFINED {
            self.queue.push(v);
            self.bubble_up();
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
        let min = self.queue.swap_remove(0);
        self.bubble_down();
        self.seen[usize::from(min)] = true;
        return min;
    }
    fn empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}
pub fn dijkstra_unwrapped<I>(
    target: Option<Vertex>,
    n: usize,
    mut vertices: I,
    edges: impl Iterator<Item = Edge>,
) -> I
where
    I: PriorityQueue,
{
    let mut count = 0.0;
    let mut ratio = 0.0;
    print!("progess {:.2}%", ratio * 100.);
    std::io::stdout().flush().unwrap();

    let mut out_edges = vec![Vec::new(); n];

    for e in edges {
        out_edges[usize::from(e.from)].push(e);
    }

    while !vertices.empty() {
        //feedback
        count += 1.0;
        let tmp = count / n as f64;
        if (tmp - ratio) > 0.001 {
            ratio = tmp;
            print!("\rprogess {:.2}%", ratio * 100.);
            std::io::stdout().flush().unwrap();
        }

        //choose next vector
        let u = vertices.pop_min();
        if target == Some(u) {
            break;
        }

        // update neighbors of u
        for e in &out_edges[usize::from(u)] {
            let alt = vertices.get_dist(e.from) + e.weight;
            if !vertices.expanded(e.to) && alt < vertices.get_dist(e.to) {
                vertices.update_vertice(e.to, alt, u);
            }
        }
    }
    println!("");
    return vertices;
}
