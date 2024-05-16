use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use paste::paste;
// use rand::{rngs::ThreadRng, thread_rng, Rng};
use sp_algo::{dijkstra::*, dimacs::*, implicit_heaps::*, pairing_heap::*};
use std::path::Path;

#[allow(dead_code)]
const SMALLER_REGIONS: [&str; 12] = [
    "USA", "CTR", "W", "E", "LKS", "CAL", "NE", "NW", "FLA", "COL", "BAY", "NY",
];

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

#[inline]
fn preprocess_graph(region: &str, n: usize) -> NeighborList {
    let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
    StructuredEdges::new(n, edges)
}

#[inline]
fn benchmark<Q>(size: usize, graph: &NeighborList)
where
    Q: PriorityQueue + HasTypeName + InitDijkstra,
{
    let queue = Q::init_dijkstra(Vertex(1), size);
    sssp(queue, graph);
}

#[inline]
fn setup(region: &str) -> (NeighborList, usize) {
    let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
    let size = n + 1;
    let graph: NeighborList = preprocess_graph(region, size);
    (graph, size)
}

macro_rules! run {
    ($Q:ident) => {
        paste! {
            #[library_benchmark]
            #[bench::in_ny(setup("NY"))]
            #[bench::in_ne(setup("NE"))]
            #[bench::in_e(setup("E"))]
            #[bench::in_usa(setup("USA"))]
            fn [<run_ $Q:lower>](input: (NeighborList, usize))  {
                let (graph, size) = input;
                benchmark::<$Q>(size, &graph);
            }
        }
    };
}

run!(BinaryHeap);
run!(PentaryHeap);
run!(OctaryHeap);
run!(HexadecimaryHeap);
run!(BinaryHeapSimple);
run!(PentaryHeapSimple);
run!(OctaryHeapSimple);
run!(HexadecimaryHeapSimple);
run!(PairingHeap);

library_benchmark_group!(
    name = sssp;
    compare_by_id = true;
    benchmarks = run_binaryheap, run_pentaryheap, run_octaryheap, run_hexadecimaryheap,
    run_binaryheapsimple, run_pentaryheapsimple, run_octaryheapsimple,
    run_hexadecimaryheapsimple,
);

main!(library_benchmark_groups = sssp);
