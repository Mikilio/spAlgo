use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};
use std::fmt::Debug;
use std::hash::BuildHasherDefault;
use std::slice::Iter;
use std::usize;

use nohash_hasher::{IsEnabled, NoHashHasher};

use crate::dimacs::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Item {
    pub key: u32,
    pub value: Vertex,
}

pub struct SimpleList {
    inner: Vec<Item>,
}

impl From<Vertex> for SimpleList {
    #[inline]
    fn from(value: Vertex) -> Self {
        let mut inner = Vec::new();
        inner.push(Item { key: 0, value });
        Self { inner }
    }
}

impl PartialOrd for Item {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.key.cmp(&self.key))
    }
}

impl Ord for Item {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.key.cmp(&self.key)
    }
}

pub struct Search<T: DecreaseKey> {
    pub queue: T,
    pub meta:
        HashMap<Vertex, (T::RefType, T::Key, T::Value), BuildHasherDefault<NoHashHasher<T::Key>>>,
}

impl<T: DecreaseKey> From<(Vertex, usize)> for Search<T> {
    #[inline]
    fn from(tuple: (Vertex, usize)) -> Self {
        let (source, size) = tuple;
        let item = (
            T::RefType::from(source),
            T::Key::from(0),
            T::Value::from(source),
        );
        let mut map = HashMap::with_capacity_and_hasher(size, BuildHasherDefault::default());
        map.insert(source, item);
        Self {
            queue: T::from(source),
            meta: map,
        }
    }
}

pub struct OwnedLookup<T: DecreaseKey> {
    pub queue: T,
    pub meta: HashMap<Vertex, (T::Key, T::Value), BuildHasherDefault<NoHashHasher<T::Key>>>,
}

impl<T: DecreaseKey> From<(Vertex, usize)> for OwnedLookup<T> {
    #[inline]
    fn from(tuple: (Vertex, usize)) -> Self {
        let (source, size) = tuple;
        let item = (0.into(), T::Value::from(source));
        let mut map = HashMap::with_capacity_and_hasher(size, BuildHasherDefault::default());
        map.insert(source, item);
        Self {
            queue: T::from(source),
            meta: map,
        }
    }
}

pub struct NoLookup<T: PriorityQueue> {
    pub queue: T,
    pub meta: HashMap<Vertex, (Option<T::Key>, T::Value), BuildHasherDefault<NoHashHasher<T::Key>>>,
}

impl<T: PriorityQueue> From<(Vertex, usize)> for NoLookup<T> {
    #[inline]
    fn from(tuple: (Vertex, usize)) -> Self {
        let (value, size) = tuple;
        let item = (None, T::Value::from(value));
        let mut map = HashMap::with_capacity_and_hasher(size, BuildHasherDefault::default());
        map.insert(value, item);
        Self {
            queue: T::from(value),
            meta: map,
        }
    }
}

pub trait PriorityQueue: From<Vertex> {
    type RefType: From<Vertex> + PartialEq + Debug + Clone;
    type Key: From<u32> + Into<u32> + IsEnabled + Eq + Debug + Copy;
    type Value: From<Vertex> + Into<Vertex> + Eq + Debug + Copy;

    fn is_empty(&self) -> bool;
    fn pop(&mut self) -> Option<(Self::Key, Self::Value)>;
    fn push(&mut self, key: Self::Key, value: Self::Value) -> Self::RefType;
}

pub trait DecreaseKey: PriorityQueue {
    fn decrease_key(&mut self, of: Self::RefType, key: Self::Key);
}

pub trait Dijkstra {
    type Queue: PriorityQueue;

    fn explore(
        &mut self,
        from: <Self::Queue as PriorityQueue>::Value,
        key: <Self::Queue as PriorityQueue>::Key,
        e: &Neighbor,
    );
    fn pop_min(
        &mut self,
    ) -> Option<(
        <Self::Queue as PriorityQueue>::Key,
        <Self::Queue as PriorityQueue>::Value,
    )>;

    fn get_meta(
        &self,
        target: Vertex,
    ) -> Option<(
        <Self::Queue as PriorityQueue>::Key,
        <Self::Queue as PriorityQueue>::Value,
    )>;

    fn get_path(&self, target: Vertex) -> Option<Vec<Vertex>> {
        let mut path = Vec::new();
        let mut head = target;
        while let Some((_, prev)) = self.get_meta(head) {
            path.push(head);
            if head != prev.into() {
                head = prev.into();
            } else {
                return Some(path);
            }
        }
        None
    }

    fn get_dist(&self, target: Vertex) -> Option<u32> {
        if let Some((dist, _)) = self.get_meta(target) {
            return Some(dist.into());
        }
        None
    }
}

pub trait InitDijkstra: PriorityQueue {
    type Data: From<(Vertex, usize)> + Dijkstra;

    #[inline]
    fn init_dijkstra(source: Vertex, size: usize) -> impl Dijkstra {
        Self::Data::from((source, size))
    }
}

impl<T: DecreaseKey> Dijkstra for Search<T> {
    type Queue = T;

    #[inline]
    fn explore(&mut self, from: T::Value, key: T::Key, e: &Neighbor) {
        let alt: u32 = key.into() + e.weight;
        let explored = self.meta.entry(e.to.into());
        match explored {
            Occupied(mut entry) => {
                let (link, dist, prev) = entry.get_mut();
                let none: T::RefType = Vertex(0).into();
                if none == *link {
                    return;
                }
                if alt < (*dist).into() {
                    self.queue.decrease_key(link.clone(), alt.into());
                    *dist = alt.into();
                    *prev = from;
                }
            }
            Vacant(entry) => {
                let link = self.queue.push(alt.into(), e.to.into());
                entry.insert((link, alt.into(), from));
            }
        }
    }

    #[inline]
    fn pop_min(&mut self) -> Option<(T::Key, T::Value)> {
        if let Some((key, value)) = self.queue.pop() {
            let (link, _k, _) = self.meta.get_mut(&value.into()).unwrap();
            *link = Vertex(0).into();
            return Some((key, value));
        }
        None
    }

    #[inline]
    fn get_meta(
        &self,
        target: Vertex,
    ) -> Option<(
        <Self::Queue as PriorityQueue>::Key,
        <Self::Queue as PriorityQueue>::Value,
    )> {
        if let Some((_, dist, prev)) = self.meta.get(&target) {
            return Some((*dist, *prev));
        }
        None
    }
}

impl<T: DecreaseKey> Dijkstra for OwnedLookup<T> {
    type Queue = T;

    #[inline]
    fn explore(&mut self, from: T::Value, key: T::Key, e: &Neighbor) {
        let alt: u32 = key.into() + e.weight;
        let explored = self.meta.entry(e.to.into());
        match explored {
            Occupied(mut entry) => {
                let (dist, prev) = entry.get_mut();
                if alt < (*dist).into() {
                    self.queue.decrease_key(e.to.into(), alt.into());
                    *dist = alt.into();
                    *prev = from;
                }
            }
            Vacant(entry) => {
                self.queue.push(alt.into(), e.to.into());
                entry.insert((alt.into(), from));
            }
        }
    }

    #[inline]
    fn pop_min(&mut self) -> Option<(T::Key, T::Value)> {
        self.queue.pop()
    }

    #[inline]
    fn get_meta(
        &self,
        target: Vertex,
    ) -> Option<(
        <Self::Queue as PriorityQueue>::Key,
        <Self::Queue as PriorityQueue>::Value,
    )> {
        self.meta.get(&target).copied()
    }
}

impl<T: PriorityQueue> Dijkstra for NoLookup<T> {
    type Queue = T;

    #[inline]
    fn explore(&mut self, from: T::Value, key: T::Key, e: &Neighbor) {
        let alt: u32 = key.into() + e.weight;
        self.queue.push(alt.into(), e.to.into());
        match self.meta.get_mut(&e.to) {
            None => {
                self.meta.insert(e.to, (None, from));
            }
            Some((None, prev)) => {
                *prev = from;
            }
            _ => (),
        }
    }

    #[inline]
    fn pop_min(&mut self) -> Option<(T::Key, T::Value)> {
        while let Some((key, value)) = self.queue.pop() {
            let (extended, _) = self.meta.get_mut(&value.into()).unwrap();
            if let Some(_) = extended {
                //skip expanded items
                continue;
            } else {
                *extended = Some(key);
                return Some((key, value));
            }
        }
        return None;
    }

    #[inline]
    fn get_meta(
        &self,
        target: Vertex,
    ) -> Option<(
        <Self::Queue as PriorityQueue>::Key,
        <Self::Queue as PriorityQueue>::Value,
    )> {
        if let Some((Some(dist), prev)) = self.meta.get(&target) {
            return Some((*dist, *prev));
        }
        None
    }
}

impl PriorityQueue for SimpleList {
    type RefType = usize;
    type Key = u32;
    type Value = Vertex;

    #[inline]
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    fn pop(&mut self) -> Option<(Self::Key, Self::Value)> {
        if let Some(res) = self.inner.pop() {
            return Some((res.key, res.value));
        }
        None
    }

    #[inline]
    fn push(&mut self, key: Self::Key, value: Self::Value) -> Self::RefType {
        let item = Item { key, value };
        match self.inner.binary_search(&item) {
            Ok(pos) => {
                self.inner.insert(pos, item);
                pos // element already in vector @ `pos`
            }
            Err(pos) => {
                self.inner.insert(pos, item);
                pos
            }
        }
    }
}

impl InitDijkstra for SimpleList {
    type Data = NoLookup<Self>;
}

#[derive(Clone, Copy, Debug)]
pub struct Neighbor {
    pub to: Vertex,
    pub weight: u32,
}

impl From<Edge> for Neighbor {
    #[inline]
    fn from(value: Edge) -> Self {
        Neighbor {
            to: value.to,
            weight: value.weight,
        }
    }
}

pub type NeighborList = Vec<Vec<Neighbor>>;

pub trait StructuredEdges {
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self;
    fn get_neighbors(&self, u: Vertex) -> Iter<Neighbor>;
}

impl StructuredEdges for NeighborList {
    #[inline]
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self {
        let mut out_edges: Vec<Vec<Neighbor>> = vec![Vec::new(); n];

        for e in edges {
            out_edges[usize::from(e.from)].push(Neighbor::from(e));
        }
        return out_edges;
    }
    #[inline]
    fn get_neighbors(&self, u: Vertex) -> Iter<Neighbor> {
        self[usize::from(u)].iter()
    }
}

#[inline]
pub fn sssp<D>(mut data: D, edges: &NeighborList) -> D
where
    D: Dijkstra,
{
    while let Some((dist, u)) = data.pop_min() {
        // update neighbors of u
        for e in edges.get_neighbors(u.into()) {
            data.explore(u, dist, e);
        }
    }
    data
}

#[cfg(test)]
mod tests {

    use colored::Colorize;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::io::Read;
    use std::io::Write;
    use std::{fs::File, path::Path};

    use crate::implicit_heaps::BinaryHeap;
    use crate::implicit_heaps::BinaryHeapSimple;
    use crate::pairing_heap::PairingHeap;

    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn push_pop_simple_list() {
        let n = 10000;
        let mut highest_min = 0;
        let mut dijkstra: NoLookup<SimpleList> = NoLookup::from((Vertex(1), n));
        let mut rng = thread_rng();
        //push
        for i in 1..n {
            let to = Vertex::try_from(i).unwrap();
            dijkstra.explore(
                Vertex(1),
                0,
                &Neighbor {
                    weight: rng.gen_range(1..1000000),
                    to,
                },
            );
        }
        //some more pushes
        for _ in 0..n {
            let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let key = rng.gen_range(1..1000000);
            dijkstra.explore(Vertex(1), 0, &Neighbor { weight: key, to });
        }
        //pop
        for _ in 0..n {
            let (key, popped) = dijkstra.pop_min().unwrap();
            let (stored_key, _) = dijkstra.meta.get(&popped).unwrap();
            assert_eq!(key, stored_key.unwrap());
            assert!(key >= highest_min);
            highest_min = u32::max(highest_min, key);
        }

        assert_eq!(None, dijkstra.pop_min());
    }

    macro_rules! sssp_test {
        // using a ty token type for macthing datatypes passed to maccro
        ($name:ident,$T:ident, $Q:ident) => {
            #[test]
            fn $name() {
                let n: usize = load_max_vertex(Path::new("./data/NY.co")).into();
                let size = n + 1;
                let edges = load_edges(Path::new("./data/NY-d.gr"));
                let graph: NeighborList = StructuredEdges::new(size, edges);
                let dijkstra: $T<$Q> = $T::from((Vertex(1), size));
                let result = sssp(dijkstra, &graph);
                let path = Path::new("./test/NY.distances");
                match File::open(path) {
                    Ok(mut f) => {
                        // Check if the file is empty
                        let mut buffer = [0u8];
                        let c = f.read(&mut buffer).unwrap();
                        if c < 1 {
                            // If the file is empty, reopen it for writing
                            let mut file = File::options().write(true).open(&path).unwrap();
                            for i in 1..size {
                                write!(
                                    file,
                                    "{}: {}\n",
                                    i,
                                    result
                                        .get_dist(Vertex(i.try_into().unwrap()))
                                        .expect(&format!("it had no distance {}", i)),
                                    // Route(result
                                    //     .get_path(Vertex(i.try_into().unwrap()))
                                    //     .expect("there is no path"))
                                )
                                .unwrap();
                                //flush from time to time so my pc can have some memory
                                if i % 1000 == 0 {
                                    file.flush().unwrap();
                                }
                            }
                        } else {
                            // If the file is not empty, reopen it for reading
                            let file = File::open(&path).unwrap();
                            let reader = BufReader::new(file);
                            let mut lines = reader.lines();
                            for i in 1..size {
                                let line = format!(
                                    "{}: {}",
                                    i,
                                    result
                                        .get_dist(Vertex(i.try_into().unwrap()))
                                        .expect(&format!("it had no distance {}", i)),
                                    // Route(result
                                    //     .get_path(Vertex(i.try_into().unwrap()))
                                    //     .expect("there is no path"))
                                );
                                assert_eq!(lines.next().unwrap().unwrap(), line);
                            }
                        }
                    }
                    Err(_) => {
                        panic!(
                            "⚠️ {}",
                            "Please prepare the tests with `prepare-tests`"
                                .bold()
                                .yellow()
                        );
                    }
                };
            }
        };
    }
    sssp_test!(sssp_test_binary, OwnedLookup, BinaryHeap);
    sssp_test!(sssp_test_pairing, Search, PairingHeap);
    sssp_test!(sssp_test_list, NoLookup, SimpleList);
    sssp_test!(sssp_test_simple, NoLookup, BinaryHeapSimple);
}
