use crate::dimacs::*;
use rand::{thread_rng, Rng};
use std::path::Path;

pub mod dimacs;

struct VertexList {
    queue: Vec<Vertex>,
    dist: Vec<u32>,
    prev: Vec<Vertex>,
}

impl VertexList {
    fn new(n: usize, source: Vertex) -> Self {
        let mut queue: Vec<Vertex> = Vec::with_capacity(n);
        let mut dist: Vec<u32> = Vec::with_capacity(n);
        let mut prev: Vec<Vertex> = Vec::with_capacity(n);

        for i in 0..n {
            let v = Vertex::try_from(i).unwrap();
            queue.push(v);
            dist.push(if v == source { 0 } else { u32::MAX });
            prev.push(UNDEFINED);
        }
        Self { queue, dist, prev }
    }
}

type VertexHeap = VertexList;

trait PriorityQueue {
    fn update_dist(&mut self, v: Vertex, new: u32);
    fn update_prev(&mut self, v: Vertex, new: Vertex);
    fn get_dist(&mut self, v: Vertex) -> u32;
    fn get_prev(&mut self, v: Vertex) -> Vertex;
    fn pop_min(&mut self) -> Vertex;
    fn contains(&self, v: &Vertex) -> bool;
    fn empty(&mut self) -> bool;
}

impl PriorityQueue for VertexList {
    fn update_dist(&mut self, v: Vertex, new: u32) {
        self.dist[usize::from(v)] = new;
    }
    fn update_prev(&mut self, v: Vertex, new: Vertex) {
        self.prev[usize::from(v)] = new;
    }
    fn get_dist(&mut self, v: Vertex) -> u32 {
        self.dist[usize::from(v)]
    }
    fn get_prev(&mut self, v: Vertex) -> Vertex {
        self.prev[usize::from(v)]
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
        return self.queue.swap_remove(i);
    }
    fn contains(&self, v: &Vertex) -> bool {
        self.queue.contains(v)
    }
    fn empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

fn dijkstra_unwrapped<I>(
    target: Option<Vertex>,
    n: usize,
    mut vertices: I,
    mut edges: Vec<Edge>,
) -> I
where
    I: PriorityQueue,
{
    let mut count = 0.0;
    let mut ratio = 0.0;

    while !vertices.empty() {
        //feedback
        count += 1.0;
        let tmp = count / n as f64;
        if (tmp - ratio) > 0.001 {
            ratio = tmp;
            println!("ratio {:.3}", ratio);
        }

        //choose next vector
        let u = vertices.pop_min();
        if target == Some(u) {
            break;
        }

        // get neighbors of u
        let (nbi, nbe): (Vec<usize>, Vec<&Edge>) = edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if e.from == u && vertices.contains(&e.to) {
                    Some((i, e))
                } else {
                    None
                }
            })
            .unzip();

        // update neighbors
        for e in nbe {
            let alt = vertices.get_dist(u) + e.weight;
            if alt < vertices.get_dist(e.to) {
                vertices.update_dist(e.to, alt);
                vertices.update_prev(e.to, u);
            }
        }
        //discard used edges
        for i in nbi.iter().rev() {
            let _ = edges.swap_remove(*i);
        }
    }
    return vertices;
}

fn main() {
    let mut rng = thread_rng();

    let n: usize = load_max_vertex(Path::new("./data/NY.co")).into();
    let source: Vertex = rng.gen_range(0..n).try_into().unwrap();
    let vertices = VertexList::new(n, source);
    let edges: Vec<Edge> = load_edges(Path::new("./data/NY-d.gr")).collect();

    let vlist = dijkstra_unwrapped(None, n, vertices, edges);
    assert_eq!(
        (
            vlist.prev[usize::from(source)],
            vlist.dist[usize::from(source)]
        ),
        (UNDEFINED, 0)
    );
}
