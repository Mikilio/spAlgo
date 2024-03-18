use crate::dimacs::*;
use rand::{thread_rng, Rng};
use std::path::Path;

pub mod dimacs;

struct PriorityQueue {
    queue: Vec<Vertex>,
    dist: Vec<u32>,
    prev: Vec<Vertex>,
}

fn pop_min(dist: &Vec<u32>, queue: &mut Vec<Vertex>) -> Vertex {
    let i = queue
        .iter()
        .enumerate()
        .min_by(|(_, &a), (_, &b)| (&dist[usize::from(a)]).cmp(&dist[usize::from(b)]))
        .map(|(index, _)| index)
        .unwrap();
    return queue.swap_remove(i);
}

pub fn dijkstra(
    source: Vertex,
    target: Option<Vertex>,
    n: usize,
    mut edges: Vec<Edge>,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut queue: Vec<Vertex> = Vec::with_capacity(n);
    let mut dist: Vec<u32> = Vec::with_capacity(n);
    let mut prev: Vec<Vertex> = Vec::with_capacity(n);

    for i in 0..n {
        let v = Vertex::try_from(i).unwrap();
        queue.push(v);
        dist.push(if v == source { 0 } else { u32::MAX });
        prev.push(UNDEFINED);
    }

    let mut count = 0.0;
    let mut ratio = 0.0;

    while !queue.is_empty() {
        //feedback
        count += 1.0;
        let tmp = count / n as f64;
        if (tmp - ratio) > 0.001 {
            ratio = tmp;
            println!("ratio {:.3}", ratio);
        }

        //choose next vector
        let u = pop_min(&dist, &mut queue);
        if target == Some(u) {
            break;
        }

        // get neighbors of u
        let (nbi, nbe): (Vec<usize>, Vec<&Edge>) = edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if e.from == u && queue.contains(&e.to) {
                    Some((i, e))
                } else {
                    None
                }
            })
            .unzip();

        // update neighbors
        for e in nbe {
            let alt = dist[usize::from(u)] + e.weight;
            if alt < dist[usize::from(e.to)] {
                dist[usize::from(e.to)] = alt;
                prev[usize::from(e.to)] = u;
            }
        }
        //discard used edges
        for i in nbi.iter().rev() {
            let _ = edges.swap_remove(*i);
        }
    }
    return (prev, dist);
}

fn main() {
    let mut rng = thread_rng();

    let n: usize = load_max_vertex(Path::new("./data/NY.co")).into();
    let source: Vertex = rng.gen_range(0..n).try_into().unwrap();
    let edges: Vec<Edge> = load_edges(Path::new("./data/NY-d.gr")).collect();

    let (ancestors, distances) = dijkstra(source, None, n, edges);
    assert_eq!(
        (
            ancestors[usize::from(source)],
            distances[usize::from(source)]
        ),
        (UNDEFINED, 0)
    );
}
