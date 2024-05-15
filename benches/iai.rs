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

macro_rules! control {
    ($region:literal) => {
        paste! {
            #[inline]
            fn [<control_ $region:lower>]() {
                let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", $region))).into();
                let size = n + 1;
                let _graph: NeighborList = preprocess_graph($region, size);
            }
        }
    };
}
macro_rules! run {
    ($region:literal, $Q:ident) => {
        paste! {
            #[inline]
            fn [<run_ $region:lower _ $Q:lower>]() {
                let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", $region))).into();
                let size = n + 1;
                let graph: NeighborList = preprocess_graph($region, size);
                benchmark::<$Q>(size, &graph);
            }
        }
    };
}

control!("NY");
run!("NY", BinaryHeap);
run!("NY", PentaryHeap);
run!("NY", OctaryHeap);
run!("NY", HexadecimaryHeap);
run!("NY", BinaryHeapSimple);
run!("NY", PentaryHeapSimple);
run!("NY", OctaryHeapSimple);
run!("NY", HexadecimaryHeapSimple);

control!("NE");
run!("NE", BinaryHeap);
run!("NE", PentaryHeap);
run!("NE", OctaryHeap);
run!("NE", HexadecimaryHeap);
run!("NE", BinaryHeapSimple);
run!("NE", PentaryHeapSimple);
run!("NE", OctaryHeapSimple);
run!("NE", HexadecimaryHeapSimple);

control!("E");
run!("E", BinaryHeap);
run!("E", PentaryHeap);
run!("E", OctaryHeap);
run!("E", HexadecimaryHeap);
run!("E", BinaryHeapSimple);
run!("E", PentaryHeapSimple);
run!("E", OctaryHeapSimple);
run!("E", HexadecimaryHeapSimple);

control!("USA");
run!("USA", BinaryHeap);
run!("USA", PentaryHeap);
run!("USA", OctaryHeap);
run!("USA", HexadecimaryHeap);
run!("USA", BinaryHeapSimple);
run!("USA", PentaryHeapSimple);
run!("USA", OctaryHeapSimple);
run!("USA", HexadecimaryHeapSimple);

iai::main!(
    control_ny,
    control_ne,
    control_e,
    control_usa,
    run_ny_binaryheap,
    run_ny_pentaryheap,
    run_ny_octaryheap,
    run_ny_hexadecimaryheap,
    run_ny_binaryheapsimple,
    run_ny_pentaryheapsimple,
    run_ny_octaryheapsimple,
    run_ny_hexadecimaryheapsimple,
    run_ne_binaryheap,
    run_ne_pentaryheap,
    run_ne_octaryheap,
    run_ne_hexadecimaryheap,
    run_ne_binaryheapsimple,
    run_ne_pentaryheapsimple,
    run_ne_octaryheapsimple,
    run_ne_hexadecimaryheapsimple,
    run_e_binaryheap,
    run_e_pentaryheap,
    run_e_octaryheap,
    run_e_hexadecimaryheap,
    run_e_binaryheapsimple,
    run_e_pentaryheapsimple,
    run_e_octaryheapsimple,
    run_e_hexadecimaryheapsimple,
    run_usa_binaryheap,
    run_usa_pentaryheap,
    run_usa_octaryheap,
    run_usa_hexadecimaryheap,
    run_usa_binaryheapsimple,
    run_usa_pentaryheapsimple,
    run_usa_octaryheapsimple,
    run_usa_hexadecimaryheapsimple,
);
