use criterion::{
    criterion_group, criterion_main, measurement::WallTime, profiler::Profiler, AxisScale,
    BenchmarkGroup, BenchmarkId, Criterion, PlotConfiguration, SamplingMode,
};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use sp_algo::{dijkstra::*, dimacs::*, implicit_heaps::*, pairing_heap::*};
use std::{fs, path::Path, process::Command, time::Duration};

struct GProfiler;

impl Profiler for GProfiler {
    fn start_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        let path = benchmark_dir.join("main.profile");
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        cpuprofiler::PROFILER
            .lock()
            .unwrap()
            .start(path.to_str().unwrap())
            .unwrap();
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        cpuprofiler::PROFILER.lock().unwrap().stop().unwrap();
        if let Ok(path) = std::env::current_exe() {
            let symbols = path.to_str().unwrap();
            let input = benchmark_dir.join("main.profile");
            let output = benchmark_dir.join("main.raw");
            let raw = Command::new("pprof")
                .args(["--raw", symbols, input.to_str().unwrap()])
                .output()
                .expect("failed to create raw profile");
            fs::write(&output, &raw.stdout).unwrap();
        }
    }
}

trait HasTypeName {
    fn type_name() -> &'static str;
}

macro_rules! impl_has_type_name {
    ($($name:ty),* $(,)?) => (
        $(
            impl HasTypeName for $name {
                #[inline]
                fn type_name() -> &'static str {
                    stringify!($name)
                }
            }
        )*
    );
}

impl_has_type_name!(
    SortetList,
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

pub fn cmp_sp_queries(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        "USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA", "COL", "BAY", "NY",
    ];
    let rng = &mut thread_rng();
    let mut group = c.benchmark_group("SP_Queries");
    group
        .measurement_time(Duration::from_secs(1000))
        .sample_size(100)
        .sampling_mode(SamplingMode::Flat)
        .plot_config(plot_config);
    for region in smaller_regions {
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let graph = preprocess_graph(region, size);
        let bigraph = preprocess_bigraph(region, size);
        group.bench_with_input(BenchmarkId::new("Naiv", &size), &size, |b, &size| {
            b.iter_batched(
                || {
                    (
                        PentaryHeap::init_dijkstra(
                            rng.gen_range(0..size).try_into().unwrap(),
                            size,
                        ),
                        rng.gen_range(0..size).try_into().unwrap(),
                    )
                },
                |(source, target)| sp_naiv(source, target, &graph),
                criterion::BatchSize::LargeInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("Bi", &size), &size, |b, &size| {
            b.iter_batched(
                || {
                    (
                        PentaryHeap::init_dijkstra(
                            rng.gen_range(0..size).try_into().unwrap(),
                            size,
                        ),
                        PentaryHeap::init_dijkstra(
                            rng.gen_range(0..size).try_into().unwrap(),
                            size,
                        ),
                    )
                },
                |(source, target)| sp_bi(source, target, &bigraph),
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

pub fn cmp_sssp(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let smaller_regions = [
        "USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA", "COL", "BAY", "NY",
    ];
    let rng = &mut thread_rng();
    let mut group = c.benchmark_group("SSSP");
    group
        .measurement_time(Duration::from_secs(100))
        .sample_size(10)
        .sampling_mode(SamplingMode::Flat)
        .plot_config(plot_config);
    for region in smaller_regions {
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let graph: NeighborList = preprocess_graph(region, size);
        benchmark::<BinaryHeap>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeap>(rng, size, &graph, &mut group);
        benchmark::<OctaryHeap>(rng, size, &graph, &mut group);
        benchmark::<HexadecimaryHeap>(rng, size, &graph, &mut group);
        benchmark::<BinaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<PentaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<OctaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<HexadecimaryHeapSimple>(rng, size, &graph, &mut group);
        benchmark::<PairingHeap>(rng, size, &graph, &mut group);
        benchmark::<SortetList>(rng, size, &graph, &mut group);
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(GProfiler);
    targets = /* cmp_sssp,*/cmp_sp_queries
}
criterion_main!(benches);

#[inline]
fn preprocess_graph(region: &str, n: usize) -> NeighborList {
    let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
    StructuredEdges::new(n, edges)
}

#[inline]
fn preprocess_bigraph(region: &str, n: usize) -> DicirectionalList<NeighborList> {
    let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
    DicirectionalList::new(n, edges)
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
