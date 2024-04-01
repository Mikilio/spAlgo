use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;
use sp_algo::{dijkstra::*, dimacs::*};
use std::path::Path;
use std::time::Duration;

pub fn compare_queues(c: &mut Criterion) {
    let smaller_regions = ["COL", "BAY", "NY"];
    let mut rng = thread_rng();
    let mut group = c.benchmark_group("Priority Queues");
    group
        .measurement_time(Duration::from_secs(1000))
        .sample_size(500);
    for region in smaller_regions {
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let graph: NeighborList = preprocess_graph(region, size);
        group.bench_with_input(
            BenchmarkId::new("Vector List", region),
            &size,
            |b, &size| {
                b.iter_batched(
                    || init::<SimpleList>(&mut rng, size),
                    |queue| dijkstra(queue, &graph),
                    criterion::BatchSize::LargeInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Binary Heap", region),
            &size,
            |b, &size| {
                b.iter_batched(
                    || init::<BinaryHeap>(&mut rng, size),
                    |queue| dijkstra(queue, &graph),
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(benches, compare_queues);
criterion_main!(benches);

#[inline]
fn preprocess_graph<E>(region: &str, n: usize) -> E
where
    E: StructuredEdges,
{
    let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
    E::new(n, edges)
}

#[inline]
fn init<Q>(rng: &mut ThreadRng, n: usize) -> Q
where
    Q: PriorityQueue,
{
    let source: Vertex = rng.gen_range(0..n).try_into().unwrap();
    Q::new(n, source)
}
