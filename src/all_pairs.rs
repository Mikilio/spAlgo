use std::{
    fs::File,
    io::{self, stdout, BufWriter, Error, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
    usize,
};

use chrono::{DateTime, Local};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::dijkstra::{DicirectionalList, OwnedLookup};
use crate::{
    dijkstra::{sssp, Dijkstra, NeighborList},
    dimacs::{CostMatrix, Vertex},
    implicit_heaps::PentaryHeap,
};

/// Size of each block for block-wise operations.
const BLOCK_SIZE: usize = 4096 * 3;
/// Nerf factor used to run test in fasible time. Set to 1 to run fill algorithm.
const NERF_FACTOR: usize = 200;

/// Wrapper struct for transposed matrix.
struct Transpose(Vec<u32>);
/// Wrapper struct for matrix.
struct Matrix(Vec<u32>);

/// Convert a slice of u32 to a slice of u8.
fn as_u8_slice(v: &[u32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            v.as_ptr() as *const u8,
            v.len() * std::mem::size_of::<u32>(),
        )
    }
}

/// Convert a mutable slice of u32 to a mutable slice of u8.
fn as_u8_slice_mut(v: &mut [u32]) -> &mut [u8] {
    unsafe {
        std::slice::from_raw_parts_mut(v.as_ptr() as *mut u8, v.len() * std::mem::size_of::<u32>())
    }
}

/// Convert graph to matrix.
fn graph2matrix(graph: &NeighborList, row: usize, col: usize) -> Vec<u32> {
    let mut matrix = vec![u32::MAX; BLOCK_SIZE * BLOCK_SIZE];
    let row_start = row * BLOCK_SIZE;
    let col_start = col * BLOCK_SIZE;
    let col_end = col_start + BLOCK_SIZE;
    for i in 0..BLOCK_SIZE {
        matrix[i * BLOCK_SIZE + i] = 0;
        for e in graph[row_start + i].iter() {
            let j = usize::from(e.to);
            if j < col_end && col_start <= j {
                matrix[i * BLOCK_SIZE + j] = e.weight;
            }
        }
    }
    matrix
}

/// Transpose a matrix.
fn transpose(a: &Matrix) -> Transpose {
    let mut b = Vec::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
    unsafe { b.set_len(BLOCK_SIZE * BLOCK_SIZE) }
    transpose::transpose(&a.0, &mut b, BLOCK_SIZE, BLOCK_SIZE);
    Transpose(b)
}

/// Cross two blocks for block-wise Warshall-Floyd algorithm.
///NOTE:this function can further be optimized by tiling
///
/// # Arguments
///
/// * `a` - Matrix for the first block.
/// * `b` - Transposed matrix of second block.
fn wf_block(a: Matrix, b: &Transpose) -> Matrix {
    let mut a = a;
    for k in 0..BLOCK_SIZE {
        for i in 0..BLOCK_SIZE {
            for j in 0..BLOCK_SIZE {
                a.0[BLOCK_SIZE * i + j] = u32::min(
                    a.0[BLOCK_SIZE * i + j],
                    a.0[BLOCK_SIZE * k + i] + b.0[BLOCK_SIZE * k + j],
                );
            }
        }
    }
    a
}

/// Calculate all-pairs shortest paths using Warshall-Floyd algorithm.
///
/// # Arguments
///
/// * `size` - Number of Vertices.
/// * `graph` - Graph represented as a directional list of neighbor lists.
/// * `dir` - Path to the directory for storing the result file.
pub fn warshall_floyd(
    size: usize,
    graph: &DicirectionalList<NeighborList>,
    dir: &Path,
) -> Result<CostMatrix, io::Error> {
    if !dir.is_dir() {
        return Err(Error::new(
            io::ErrorKind::InvalidInput,
            "apsp expects a dir",
        ));
    }
    let current_local: DateTime<Local> = Local::now();
    let timestamp = current_local.format("%Y%m%d%H%M%S");

    let ref file_name = format!("{}/costmatrix_{}", dir.to_str().unwrap(), timestamp);
    let file_name = Path::new(file_name);

    let (mut num_blocks, rem) = (size / BLOCK_SIZE, size % BLOCK_SIZE);
    if rem > 0 {
        num_blocks = num_blocks + 1;
    }
    let num_blocks = num_blocks;
    dbg!(num_blocks * num_blocks);

    //init swaps
    let mut swaps: Vec<Mutex<Option<File>>> = Vec::with_capacity(num_blocks * num_blocks);
    for _ in 0..swaps.capacity() {
        swaps.push(Mutex::new(None));
    }
    for i in 0..num_blocks {
        for j in 0..num_blocks {
            *swaps[num_blocks * i + j].lock().unwrap() = Some(tempfile::tempfile().unwrap());
        }
    }

    //first round (k = 0)
    {
        let wkkt;
        {
            let mut wkk = Matrix(graph2matrix(&graph.forward, 0, 0));
            wkkt = transpose(&wkk);
            wkk = wf_block(wkk, &wkkt);
            dbg!("ok");
            swaps[0]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .write_all(as_u8_slice(&wkk.0))
                .unwrap();
        }

        (1..num_blocks).into_par_iter().for_each(|j| {
            dbg!(j);
            let mut wkj = Matrix(graph2matrix(&graph.forward, 0, j));
            wkj = wf_block(wkj, &wkkt);
            swaps[j]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .write_all(as_u8_slice(&wkj.0))
                .unwrap();
        });

        (1..num_blocks).into_par_iter().for_each(|i| {
            let mut buf = Vec::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
            unsafe { buf.set_len(BLOCK_SIZE * BLOCK_SIZE) }
            dbg!(i);
            let mut wik = Matrix(graph2matrix(&graph.forward, i, 0));
            wik = wf_block(wik, &wkkt);
            swaps[i * num_blocks]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .write_all(as_u8_slice(&wik.0))
                .unwrap();

            for j in 1..num_blocks {
                swaps[j]
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .read_exact(as_u8_slice_mut(buf.as_mut_slice()))
                    .unwrap();
                let wkj = Matrix(buf);
                let wikt = transpose(&wik);
                let wij = wf_block(wkj, &wikt);
                swaps[i * num_blocks + j]
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .write_all(as_u8_slice(&wij.0))
                    .unwrap();
                buf = wij.0;
            }
        });
    }

    for k in 0..num_blocks {
        dbg!(k);
        let mut buf = Vec::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
        unsafe { buf.set_len(BLOCK_SIZE * BLOCK_SIZE) }
        swaps[k * num_blocks + k]
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .read_exact(as_u8_slice_mut(buf.as_mut_slice()))
            .unwrap();
        let mut wkk = Matrix(buf);
        let ref wkkt = transpose(&wkk);
        wkk = wf_block(wkk, wkkt);
        swaps[0]
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .write_all(as_u8_slice(&wkk.0))
            .unwrap();

        (1..num_blocks).into_par_iter().for_each(|j| {
            let mut buf = Vec::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
            unsafe { buf.set_len(BLOCK_SIZE * BLOCK_SIZE) }
            swaps[k * num_blocks + j]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .read_exact(as_u8_slice_mut(buf.as_mut_slice()))
                .unwrap();
            let mut wkj = Matrix(buf);
            wkj = wf_block(wkj, wkkt);
            swaps[j]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .write_all(as_u8_slice(&wkj.0))
                .unwrap();
        });

        (1..num_blocks).into_par_iter().for_each(|i| {
            let mut buf = Vec::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
            unsafe { buf.set_len(BLOCK_SIZE * BLOCK_SIZE) }
            let wik = Vec::with_capacity(BLOCK_SIZE * BLOCK_SIZE);
            swaps[i * num_blocks + k]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .read_exact(as_u8_slice_mut(buf.as_mut_slice()))
                .unwrap();
            let mut wik = Matrix(wik);
            wik = wf_block(wik, wkkt);
            swaps[i * num_blocks]
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .write_all(as_u8_slice(&wik.0))
                .unwrap();

            for j in 1..num_blocks {
                swaps[j]
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .read_exact(as_u8_slice_mut(buf.as_mut_slice()))
                    .unwrap();
                let wkj = Matrix(buf);
                let ref wikt = transpose(&wik);
                let wij = wf_block(wkj, wikt);
                swaps[i * num_blocks + j]
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .write_all(as_u8_slice(&wij.0))
                    .unwrap();
                buf = wij.0;
            }
        });
    }

    {
        let mut wtr = BufWriter::new(File::create(&file_name)?);
        let mut buf = Vec::with_capacity(BLOCK_SIZE);
        unsafe { buf.set_len(BLOCK_SIZE * BLOCK_SIZE) }
        for b_i in 0..num_blocks {
            for r in 0..BLOCK_SIZE {
                for b_j in 0..num_blocks {
                    swaps[b_i * num_blocks + b_j]
                        .lock()
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .read_exact(as_u8_slice_mut(buf.as_mut_slice()))
                        .unwrap();
                    let index = ((b_i * num_blocks + b_j) * BLOCK_SIZE + num_blocks * r)
                        * BLOCK_SIZE
                        * std::mem::size_of::<u32>();
                    wtr.seek(SeekFrom::Start(index as u64)).unwrap();
                    wtr.write_all(as_u8_slice(&buf)).unwrap();
                    if r == (BLOCK_SIZE - 1) {
                        *swaps[b_i * num_blocks + b_j].lock().unwrap() = None;
                    }
                }
            }
        }
    }
    CostMatrix::new(file_name, size)
}

/// Calculate all-pairs shortest paths using Dijkstra's algorithm.
///
/// # Arguments
///
/// * `size` - Number of vertices.
/// * `graph` - Graph represented as a neighbor list.
/// * `dir` - Path to the directory for storing the result file.
pub fn apsp(size: usize, graph: &NeighborList, dir: &Path) -> Result<CostMatrix, io::Error> {
    if !dir.is_dir() {
        return Err(Error::new(
            io::ErrorKind::InvalidInput,
            "apsp expects a dir",
        ));
    }
    let count = AtomicU32::from(0);
    let current_local: DateTime<Local> = Local::now();
    let timestamp = current_local.format("%Y%m%d%H%M%S");

    let ref file_name = format!("{}/costmatrix_{}", dir.to_str().unwrap(), timestamp);
    let file_name = Path::new(file_name);
    {
        let wtr = Mutex::new(BufWriter::new(File::create(&file_name)?));
        let _ = (0..(size / NERF_FACTOR)).into_par_iter().for_each(|row| {
            let source: OwnedLookup<PentaryHeap> =
                OwnedLookup::from((row.try_into().unwrap(), size));
            let result = sssp(source, &graph);
            let record: Vec<u32> = (0..size)
                .map(move |i| {
                    let v: Vertex = i.try_into().unwrap();
                    result.get_dist(v).unwrap()
                })
                .collect();

            //lock
            {
                let mut lock = wtr.lock().unwrap();

                let _ = lock
                    .seek(SeekFrom::Start(
                        (row * size * std::mem::size_of::<u32>()) as u64,
                    ))
                    .unwrap();
                lock.write_all(as_u8_slice(&record)).unwrap();
            }
            //keep calm ☕
            let status = count.fetch_add(1, Ordering::Relaxed) + 1;
            if status % 100 == 0 {
                let n: u32 = size.try_into().unwrap();
                let ratio = f64::from(status) / f64::from(n / NERF_FACTOR as u32) * 100.;
                print!("processed {:.3}%\r", ratio);
                stdout().flush().unwrap();
            }
        });
        CostMatrix::new(file_name, size)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        all_pairs::{apsp, warshall_floyd, NERF_FACTOR},
        dijkstra::{DicirectionalList, NeighborList, StructuredEdges},
        dimacs::{load_edges, load_max_vertex, Vertex},
    };

    #[test]
    fn apsp_dijkstra_test() {
        let region = "NY";
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
        let graph: NeighborList = StructuredEdges::new(size, edges);
        let ref dir = Path::new("./test");

        let cost = apsp(size, &graph, dir).unwrap();

        let n: u32 = size.try_into().unwrap();
        dbg!(n);
        for node in 1u32..(n / NERF_FACTOR as u32) {
            assert_eq!(cost.get(Vertex(node), Vertex(node)).unwrap(), 0);
        }
    }

    // disabled because it takes just too long
    //#[test]
    #[allow(dead_code)]
    fn apsp_wf_test() {
        let region = "NY";
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
        let graph: DicirectionalList<NeighborList> = DicirectionalList::new(size, edges);
        let ref dir = Path::new("./test");

        let cost = warshall_floyd(size, &graph, dir).unwrap();

        let n: u32 = size.try_into().unwrap();
        dbg!(n);
        for node in 1u32..(n / NERF_FACTOR as u32) {
            assert_eq!(cost.get(Vertex(node), Vertex(node)).unwrap(), 0);
        }
    }
}
