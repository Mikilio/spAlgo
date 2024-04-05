use criterion::measurement::WallTime;
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkGroup, BenchmarkId, Criterion,
    PlotConfiguration,
};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;
use sp_algo::{dijkstra::*, dimacs::*, implicit_heaps::*};
use std::path::Path;
use std::time::Duration;

trait HasTypeName {
    fn type_name() -> &'static str;
}

macro_rules! impl_has_type_name {
    ($($name:ty),* $(,)?) => (
        $(
            impl HasTypeName for $name {
                fn type_name() -> &'static str {
                    stringify!($name)
                }
            }
        )*
    );
}

impl_has_type_name!(
    SimpleList,
    BinaryHeap,
    PentaryHeap,
    OctaryHeap,
    HexadecimaryHeap,
    BinaryHeapSimple,
    PentaryHeapSimple,
    OctaryHeapSimple,
    HexadecimaryHeapSimple
);

pub fn cmp_heap_list(c: &mut Criterion) {
    let smaller_regions = ["COL", "BAY", "NY"];
    let rng = &mut thread_rng();
    let mut group = c.benchmark_group("List vs Heap");
    group
        .measurement_time(Duration::from_secs(1000))
        .sample_size(500);
    for region in smaller_regions {
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let graph: NeighborList = preprocess_graph(region, size);
        benchmark::<SimpleList, NeighborList, Vertex, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<BinaryHeap, NeighborList, Vertex, Neighbor>(rng, size, &graph, &mut group);
    }
    group.finish();
}

pub fn cmp_heaps(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        "USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA", "COL", "BAY", "NY",
    ];
    let rng = &mut thread_rng();
    let mut group = c.benchmark_group("Heaps");
    group
        .measurement_time(Duration::from_secs(60))
        .sample_size(10)
        .plot_config(plot_config);
    for region in smaller_regions {
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let graph: NeighborList = preprocess_graph(region, size);
        benchmark::<BinaryHeap, NeighborList, Vertex, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeap, NeighborList, Vertex, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<OctaryHeap, NeighborList, Vertex, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<HexadecimaryHeap, NeighborList, Vertex, Neighbor>(
            rng, size, &graph, &mut group,
        );
    }
    group.finish();
}

pub fn cmp_simple_heaps(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        "USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA", "COL", "BAY", "NY",
    ];
    let rng = &mut thread_rng();
    let mut group = c.benchmark_group("Heaps");
    group
        .measurement_time(Duration::from_secs(60))
        .sample_size(10)
        .plot_config(plot_config);
    for region in smaller_regions {
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let graph: NeighborList = preprocess_graph(region, size);
        benchmark::<BinaryHeapSimple, NeighborList, Item, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeapSimple, NeighborList, Item, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<OctaryHeapSimple, NeighborList, Item, Neighbor>(rng, size, &graph, &mut group);
        benchmark::<HexadecimaryHeapSimple, NeighborList, Item, Neighbor>(
            rng, size, &graph, &mut group,
        );
    }
    group.finish();
}

criterion_group!(benches, cmp_simple_heaps);
criterion_main!(benches);

#[inline]
fn preprocess_graph<S, E>(region: &str, n: usize) -> S
where
    S: StructuredEdges<E>,
{
    let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
    S::new(n, edges)
}

#[inline]
fn benchmark<Q, S, I, E>(
    rng: &mut ThreadRng,
    size: usize,
    graph: &S,
    group: &mut BenchmarkGroup<WallTime>,
) where
    Q: PriorityQueue<I, E> + HasTypeName,
    S: StructuredEdges<E>,
    I: Copy,
    Vertex: From<I>,
{
    group.bench_with_input(
        BenchmarkId::new(format!("{}", Q::type_name()), &size),
        &size,
        |b, &size| {
            b.iter_batched(
                || Q::new(size, rng.gen_range(0..size).try_into().unwrap()),
                |queue| dijkstra(queue, graph),
                criterion::BatchSize::LargeInput,
            );
        },
    );
}
