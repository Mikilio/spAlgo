#[cfg(test)]
mod tests {
    use rayon::iter::ParallelBridge;
    use std::{
        io::{stdout, Write},
        iter::once,
        path::Path,
        sync::{
            atomic::{AtomicU32, Ordering},
            Mutex,
        },
        thread,
    };

    use chrono::{DateTime, Local};
    use rayon::iter::ParallelIterator;

    use crate::{
        dijkstra::{sssp, Dijkstra, NeighborList, OwnedLookup, StructuredEdges},
        dimacs::{load_edges, load_max_vertex, Vertex},
        implicit_heaps::PentaryHeap,
    };

    #[test]
    fn csv_chunk() {
        let count = AtomicU32::from(0);
        let current_local: DateTime<Local> = Local::now();
        let timestamp = current_local.format("%Y%m%d%H%M%S");
        let region = "NY";
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
        let graph: NeighborList = StructuredEdges::new(size, edges);

        let file_name = format!("./test/generic_{}_{}", region, timestamp);
        let wtr = Mutex::new(csv::Writer::from_path(file_name).unwrap());
        let rows: Vec<usize> = (1..size).collect();
        let _ = rows
            .chunks(100)
            .enumerate()
            .par_bridge()
            .for_each(|(i, chunk)| {
                let records = chunk.into_iter().map(|&row| {
                    let record = once(row.to_string());
                    let source: OwnedLookup<PentaryHeap> =
                        OwnedLookup::from((Vertex(row.try_into().unwrap()), size));
                    let result = sssp(source, &graph);
                    let dists = (1..size).map(move |col| {
                        let i: u32 = col.try_into().unwrap();
                        result.get_dist(Vertex(i)).unwrap().to_string()
                    });
                    let record = record.chain(dists);
                    record.into_iter()
                });
                //write records in order
                while i > count.load(Ordering::SeqCst) as usize {
                    thread::yield_now();
                }
                //lock
                {
                    let mut lock = wtr.lock().unwrap();
                    records.for_each(|record| lock.write_record(record).unwrap());
                }
                //keep calm ☕
                let status = count.fetch_add(1, Ordering::SeqCst) + 1;
                let n: u32 = size.try_into().unwrap();
                let ratio = f64::from(status) / f64::from(n) * 10000.;
                print!("processed {:.3}%\r", ratio);
                stdout().flush().unwrap();
            });
    }
    #[test]
    fn csv_writing() {
        let count = AtomicU32::from(1);
        let current_local: DateTime<Local> = Local::now();
        let timestamp = current_local.format("%Y%m%d%H%M%S");
        let region = "NY";
        let n: usize = load_max_vertex(Path::new(&format!("./data/{}.co", region))).into();
        let size = n + 1;
        let edges = load_edges(Path::new(&format!("./data/{}-d.gr", region)));
        let graph: NeighborList = StructuredEdges::new(size, edges);

        let file_name = format!("./test/generic_{}_{}", region, timestamp);
        let wtr = Mutex::new(csv::Writer::from_path(file_name).unwrap());
        (1..size).into_iter().par_bridge().for_each(|row| {
            let record = once(row.to_string());
            let source: OwnedLookup<PentaryHeap> =
                OwnedLookup::from((Vertex(row.try_into().unwrap()), size));
            let result = sssp(source, &graph);
            let dists = (1..size).map(move |col| {
                let i: u32 = col.try_into().unwrap();
                result.get_dist(Vertex(i)).unwrap().to_string()
            });
            let record = record.chain(dists);
            //write records in order
            while row > count.load(Ordering::SeqCst) as usize {
                thread::yield_now();
            }
            //lock
            wtr.lock().unwrap().write_record(record).unwrap();
            //keep calm ☕
            let status = count.fetch_add(1, Ordering::SeqCst) + 1;
            if status % 100 == 0 {
                let n: u32 = size.try_into().unwrap();
                let ratio = f64::from(status) / f64::from(n) * 100.;
                print!("processed {:.3}%\r", ratio);
                stdout().flush().unwrap();
            }
        });
    }
}
