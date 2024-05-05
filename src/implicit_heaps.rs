use crate::dijkstra::{DecreaseKey, InitDijkstra, Item, NoLookup, OwnedLookup, PriorityQueue};
use crate::dimacs::*;
use macros::PriorityQueue;
use nohash_hasher::NoHashHasher;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

// The hashmap resizes itself but we chose a good default that works for our use case.
const DEFAULT_SIZE: usize = 8192;

macro_rules! implicit_heap_simple {
    ($k:expr, $T:ident) => {
        // Define a priority queue struct using the given identifier ($T).
        // This queue uses a simple d_ary heap implementation.
        #[derive(PriorityQueue)]
        pub struct $T {
            inner: Vec<Item>,
        }

        impl From<Vertex> for $T {
            #[inline]
            fn from(value: Vertex) -> Self {
                // The hashmap resizes itself but we chose a good default that works for our use case.
                let mut inner = Vec::with_capacity(DEFAULT_SIZE);
                inner.push(Item { key: 0, value });
                Self { inner }
            }
        }

        impl InitDijkstra for $T {
            type Data = NoLookup<Self>;
        }

        impl $T {
            // Move the item at the specified index up in the heap
            #[inline]
            fn bubble_up(&mut self, dirt: usize) {
                let mut child = dirt;

                let mut parent;
                while child > 0 {
                    let heap = &mut self.inner;
                    parent = (child - 1) / $k;
                    let p_item = heap[parent];
                    let c_item = heap[child];
                    if p_item.key <= c_item.key {
                        break;
                    }
                    heap.swap(parent, child);
                    child = parent;
                }
            }

            // Move the root item down in the heap
            #[inline]
            fn bubble_down(&mut self) {
                let mut parent = 0;
                let n = self.inner.len();

                let mut child;
                while {
                    let heap = &mut self.inner;
                    let base = parent * $k + 1;
                    let end = usize::min(base + $k, n);
                    child = (base..end).reduce(|left, right| {
                        let l_item = heap[left];
                        let r_item = heap[right];
                        if l_item.key > r_item.key {
                            right
                        } else {
                            left
                        }
                    });
                    child != None
                } {
                    let child = child.unwrap();
                    let heap = &self.inner;
                    let p_item = heap[parent];
                    let c_item = heap[child];
                    if p_item.key <= c_item.key {
                        break;
                    }
                    self.inner.swap(parent, child);
                    parent = child;
                }
            }
        }
    };
}

macro_rules! implicit_heap {
    ($k:expr, $T:ident) => {
        // Define a priority queue struct using the given identifier ($T).
        // This queue uses a d_ary heap implementation with lookup.
        #[derive(PriorityQueue)]
        pub struct $T {
            inner: Vec<Item>,
            lookup: HashMap<Vertex, usize, BuildHasherDefault<NoHashHasher<u32>>>,
        }

        impl From<Vertex> for $T {
            #[inline]
            fn from(value: Vertex) -> Self {
                //the hashmap resizes itself but we chose a good default that works for our
                let size = DEFAULT_SIZE;
                let mut inner = Vec::with_capacity(size);
                let mut lookup =
                    HashMap::with_capacity_and_hasher(size, BuildHasherDefault::default());
                inner.push(Item { key: 0, value });
                lookup.insert(value, 0);
                Self { inner, lookup }
            }
        }

        impl InitDijkstra for $T {
            type Data = OwnedLookup<Self>;
        }

        impl DecreaseKey for $T {
            #[inline]
            fn decrease_key(&mut self, of: Self::RefType, key: Self::Key) {
                let index = self.lookup.get(&of).unwrap();
                let item = &mut self.inner[*index];
                item.key = key;
                self.bubble_up(*index);
            }
        }

        impl $T {
            // Move the item at the specified index up in the heap
            #[inline]
            fn bubble_up(&mut self, dirt: usize) {
                let mut child = dirt;

                let mut parent;
                while child > 0 {
                    let heap = &self.inner;
                    parent = (child - 1) / $k;
                    let p_item = heap[parent];
                    let c_item = heap[child];
                    if p_item.key <= c_item.key {
                        break;
                    }
                    *self.lookup.get_mut(&p_item.value).unwrap() = child;
                    *self.lookup.get_mut(&c_item.value).unwrap() = parent;
                    self.inner.swap(parent, child);
                    child = parent;
                }
            }

            // Move the root item down in the heap
            #[inline]
            fn bubble_down(&mut self) {
                let mut parent = 0;
                let n = self.inner.len();

                let mut child;
                while {
                    let heap = &self.inner;
                    let base = parent * $k + 1;
                    let end = usize::min(base + $k, n);
                    child = (base..end).reduce(|left, right| {
                        let l_item = &heap[left];
                        let r_item = &heap[right];
                        if l_item.key > r_item.key {
                            right
                        } else {
                            left
                        }
                    });
                    child != None
                } {
                    let heap = &self.inner;
                    let child = child.unwrap();
                    let p_item = &heap[parent];
                    let c_item = &heap[child];
                    if p_item.key <= c_item.key {
                        break;
                    }
                    *self.lookup.get_mut(&p_item.value).unwrap() = child;
                    *self.lookup.get_mut(&c_item.value).unwrap() = parent;
                    self.inner.swap(parent, child);
                    parent = child;
                }
            }
        }
    };
}

implicit_heap_simple!(2, BinaryHeapSimple);
implicit_heap_simple!(4, PentaryHeapSimple);
implicit_heap_simple!(8, OctaryHeapSimple);
implicit_heap_simple!(16, HexadecimaryHeapSimple);

implicit_heap!(2, BinaryHeap);
implicit_heap!(4, PentaryHeap);
implicit_heap!(8, OctaryHeap);
implicit_heap!(16, HexadecimaryHeap);

#[cfg(test)]
mod tests {

    use crate::dijkstra::*;
    use crate::implicit_heaps::*;
    use rand::thread_rng;
    use rand::Rng;

    macro_rules! push_pop_test {
        // using a ty token type for macthing datatypes passed to maccro
        ($name:ident,$T:ident) => {
            #[test]
            fn $name() {
                let n = 10000;
                let mut highest_min = 0;
                let mut dijkstra: OwnedLookup<$T> = OwnedLookup::from((Vertex(1), n));
                let mut rng = thread_rng();
                //Push
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
                //Decrease_key
                for _ in 0..n {
                    let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
                    let (key, _) = dijkstra.meta.get(&to).unwrap();
                    let key = key / 2;
                    dijkstra.explore(Vertex(1), 0, &Neighbor { weight: key, to });
                }
                //Pop
                for _ in 0..n {
                    let (key, popped) = dijkstra.pop_min().unwrap();
                    let (stored_key, _) = dijkstra.meta.get(&popped).unwrap();
                    assert_eq!(key, *stored_key);
                    assert!(key >= highest_min);
                    highest_min = u32::max(highest_min, key);
                }
                assert_eq!(dijkstra.pop_min(), None);
            }
        };
    }
    push_pop_test!(push_pop_2, BinaryHeap);
    push_pop_test!(push_pop_4, PentaryHeap);
    push_pop_test!(push_pop_8, OctaryHeap);
    push_pop_test!(push_pop_16, HexadecimaryHeap);

    macro_rules! push_pop_test_simple {
        // using a ty token type for macthing datatypes passed to maccro
        ($name:ident,$T:ident) => {
            #[test]
            fn $name() {
                let n = 10000;
                let mut highest_min = 0;
                let mut dijkstra: NoLookup<SortetList> = NoLookup::from((Vertex(1), n));
                let mut rng = thread_rng();
                //Push
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
                //Some more pushes
                for _ in 0..n {
                    let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
                    let key = rng.gen_range(1..1000000);
                    dijkstra.explore(Vertex(1), 0, &Neighbor { weight: key, to });
                }
                //Pop
                for _ in 0..n {
                    let (key, popped) = dijkstra.pop_min().unwrap();
                    let (stored_key, _) = dijkstra.meta.get(&popped).unwrap();
                    assert_eq!(key, stored_key.unwrap());
                    assert!(key >= highest_min);
                    highest_min = u32::max(highest_min, key);
                }

                assert_eq!(None, dijkstra.pop_min());
            }
        };
    }
    push_pop_test_simple!(push_pop_2_simple, BinaryHeapSimple);
    push_pop_test_simple!(push_pop_4_simple, PentaryHeapSimple);
    push_pop_test_simple!(push_pop_8_simple, OctaryHeapSimple);
    push_pop_test_simple!(push_pop_16_simple, HexadecimaryHeapSimple);
}
