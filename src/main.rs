use crate::{dijkstra::*, dimacs::*};
use rand::{thread_rng, Rng};
use std::path::Path;

pub mod dijkstra;
pub mod dimacs;

fn main() {
    let mut rng = thread_rng();

    let n: usize = load_max_vertex(Path::new("./data/NY.co")).into();
    let n = n + 1;
    println!("{}", n);
    let source: Vertex = rng.gen_range(0..n).try_into().unwrap();
    let vertices = VertexList::new(n, source);
    let edges = load_edges(Path::new("./data/NY-d.gr"));
    let vlist = dijkstra_unwrapped(None, n, vertices, edges);
    let vertices = VertexHeap::new(n, source);
    let edges = load_edges(Path::new("./data/NY-d.gr"));
    let vheap = dijkstra_unwrapped(None, n, vertices, edges);

    for i in 0..n {
        assert_eq!(vlist.dist[i], vheap.dist[i], "dist on {}", i);
        assert_eq!(vlist.prev[i], vheap.prev[i], "prev on {}", i);
    }
}
