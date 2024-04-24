use criterion::measurement::WallTime;
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkGroup, BenchmarkId, Criterion,
    PlotConfiguration,
};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;
use sp_algo::{dijkstra::*, dimacs::*, implicit_heaps::*, pairing_heap::*};
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
    HexadecimaryHeapSimple,
    PairingHeap,
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
        benchmark::<SimpleList>(rng, size, &graph, &mut group);
        benchmark::<BinaryHeap>(rng, size, &graph, &mut group);
    }
    group.finish();
}

pub fn cmp_implicit_heaps(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        /*"USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA",*/ "COL", "BAY", "NY",
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
        benchmark::<BinaryHeap>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeap>(rng, size, &graph, &mut group);
        benchmark::<OctaryHeap>(rng, size, &graph, &mut group);
        benchmark::<HexadecimaryHeap>(rng, size, &graph, &mut group);
    }
    group.finish();
}

pub fn cmp_heaps(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        /*"USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA",*/ "COL", "BAY", "NY",
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
        benchmark::<PentaryHeap>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<PairingHeap>(rng, size, &graph, &mut group);
    }
    group.finish();
}

pub fn cmp_simple_heaps(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        /*"USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA",*/ "COL", "BAY", "NY",
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
        benchmark::<BinaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<OctaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<HexadecimaryHeapSimple>(rng, size, &graph, &mut group);
    }
    group.finish();
}

criterion_group!(benches, cmp_heaps);
criterion_main!(benches);

#[inline]
fn preprocess_graph(region: &str, n: usize) -> NeighborList {
    let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
    StructuredEdges::new(n, edges)
}

#[inline]
fn benchmark<Q>(
    rng: &mut ThreadRng,
    size: usize,
    graph: &NeighborList,
    group: &mut BenchmarkGroup<WallTime>,
) where
    Q: PriorityQueue + HasTypeName + InitDijkstra,
{
    group.bench_with_input(
        BenchmarkId::new(format!("{}", Q::type_name()), &size),
        &size,
        |b, &size| {
            b.iter_batched(
                || Q::init_dijkstra(rng.gen_range(0..size).try_into().unwrap(), size),
                |queue| sssp(queue, graph),
                criterion::BatchSize::LargeInput,
            );
        },
    );
}
