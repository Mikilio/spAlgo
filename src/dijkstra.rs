use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};
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
    fn from(value: Vertex) -> Self {
        let mut inner = Vec::new();
        inner.push(Item { key: 0, value });
        Self { inner }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.key.cmp(&self.key))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.key.cmp(&self.key)
    }
}

pub struct Search<T: DecreaseKey> {
    pub queue: T,
    pub meta:
        HashMap<Vertex, (T::RefType, T::Key, T::Value), BuildHasherDefault<NoHashHasher<T::Key>>>,
}

impl<T: DecreaseKey> Search<T> {
    pub fn new(n: usize, source: Vertex) -> Self {
        let item = (
            T::RefType::from(source),
            T::Key::from(0),
            T::Value::from(source),
        );
        let mut map = HashMap::with_capacity_and_hasher(n, BuildHasherDefault::default());
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

impl<T: DecreaseKey> OwnedLookup<T> {
    pub fn new(n: usize, source: Vertex) -> Self {
        let item = (0.into(), T::Value::from(source));
        let mut map = HashMap::with_capacity_and_hasher(n, BuildHasherDefault::default());
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

impl<T: PriorityQueue> NoLookup<T> {
    pub fn new(n: usize, source: Vertex) -> Self {
        let item = (None, T::Value::from(source));
        let mut map = HashMap::with_capacity_and_hasher(n, BuildHasherDefault::default());
        map.insert(source, item);
        Self {
            queue: T::from(source),
            meta: map,
        }
    }
}

pub trait PriorityQueue: From<Vertex> {
    type RefType: From<Vertex> + Clone;
    type Key: From<u32> + Into<u32> + IsEnabled + Copy;
    type Value: From<Vertex> + Into<Vertex> + Copy;

    fn is_empty(&self) -> bool;
    fn pop(&mut self) -> (Self::Key, Self::Value);
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
    ) -> (
        <Self::Queue as PriorityQueue>::Key,
        <Self::Queue as PriorityQueue>::Value,
    );
    fn is_empty(&mut self) -> bool;
}

impl<T: DecreaseKey> Dijkstra for Search<T> {
    type Queue = T;

    fn explore(&mut self, from: T::Value, key: T::Key, e: &Neighbor) {
        let alt: u32 = key.into() + e.weight;
        let explored = self.meta.entry(e.to.into());
        match explored {
            Occupied(mut entry) => {
                let (link, dist, prev) = entry.get_mut();
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

    fn pop_min(&mut self) -> (T::Key, T::Value) {
        self.queue.pop()
    }
    fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

impl<T: DecreaseKey> Dijkstra for OwnedLookup<T> {
    type Queue = T;

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

    fn pop_min(&mut self) -> (T::Key, T::Value) {
        self.queue.pop()
    }

    fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

impl<T: PriorityQueue> Dijkstra for NoLookup<T> {
    type Queue = T;

    fn explore(&mut self, from: T::Value, key: T::Key, e: &Neighbor) {
        let alt: u32 = key.into() + e.weight;
        self.queue.push(alt.into(), e.to.into());
        self.meta.insert(e.to, (None, from));
    }

    fn pop_min(&mut self) -> (T::Key, T::Value) {
        let (mut key, mut value);
        while !self.is_empty() {
            (key,value) = self.queue.pop();
            let (extended, _) = self.meta.get_mut(&value.into()).unwrap();
            if let Some(_) = extended {
                //skip expanded items
                continue;
            } else {
                *extended = Some(key);
                return (key,value);
            }
        }
        //dummy
        return (0.into(), Vertex(0).into())
    }
    fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

impl PriorityQueue for SimpleList {
    type RefType = usize;
    type Key = u32;
    type Value = Vertex;

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn pop(&mut self) -> (Self::Key, Self::Value) {
        let res = self.inner.pop().unwrap();
        (res.key, res.value)
    }

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

#[derive(Clone, Copy, Debug)]
pub struct Neighbor {
    pub to: Vertex,
    pub weight: u32,
}

impl From<Edge> for Neighbor {
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
    fn new(n: usize, edges: impl Iterator<Item = Edge>) -> Self {
        let mut out_edges: Vec<Vec<Neighbor>> = vec![Vec::new(); n];

        for e in edges {
            out_edges[usize::from(e.from)].push(Neighbor::from(e));
        }
        return out_edges;
    }
    fn get_neighbors(&self, u: Vertex) -> Iter<Neighbor> {
        self[usize::from(u)].iter()
    }
}

pub fn sssp<D>(mut data: D, edges: &NeighborList) -> D
where
    D: Dijkstra,
{
    while !data.is_empty() {
        //choose next vector
        let (dist, u) = data.pop_min();

        // update neighbors of u
        for e in edges.get_neighbors(u.into()) {
            data.explore(u, dist, e);
        }
    }
    return data;
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn push_pop_simple_list() {
        let n = 10000;
        let mut highest_min = 0;
        let mut dijkstra: NoLookup<SimpleList> = NoLookup::new(n, Vertex(1));
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
        for i in 0..n {
            let (key, popped) = dijkstra.pop_min();
            let (stored_key, _) = dijkstra
                .meta
                .get(&popped)
                .unwrap();
            assert_eq!(key, stored_key.unwrap());
            println!("x={} on n = {}",key, i);
            assert!(key >= highest_min);
            highest_min = u32::max(highest_min, key);
        }

        assert_eq!((0,Vertex(0)),dijkstra.pop_min());
        assert!(dijkstra.is_empty());
    }
}
