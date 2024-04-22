use nohash_hasher::{IsEnabled, NoHashHasher};
use std::{
    cell::RefCell,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    hash::BuildHasherDefault,
    rc::Rc,
};

use crate::{dijkstra::Neighbor, dimacs::Vertex};

type Link<T> = Option<Rc<RefCell<T>>>;

#[derive(Debug)]
struct Node {
    id: Vertex,
    key: u32,
    parent: Link<Node>,
    child: Link<Node>,
    next: Link<Node>,
}

impl From<Vertex> for Link<Node> {
    fn from(value: Vertex) -> Self {
        Some(Rc::new(RefCell::new( Node {
            id: value,
            key: 0,
            parent: None,
            child: None,
            next: None,
        })))
    }
}

#[derive(Debug)]
struct PairingHeap {
    main: Link<Node>,
    aux: Link<Node>,
}

impl From<Vertex> for PairingHeap {
    fn from(value: Vertex) -> Self {
        Self {
            main: Link::from(value),
            aux: None,
        }
    }
}

impl PriorityQueue for PairingHeap {
    type RefType = Link<Node>;

    type Key = u32;

    type Value = Vertex;

    fn is_empty(&self) -> bool {
        if let None = self.main {
            true
        } else {
            false
        }
    }

    fn pop(&mut self) -> (Self::Key, Self::Value) {
        let aux_joined = multipass(self.aux.clone());
        self.aux = None;
        //can only panic if empty
        self.main.clone().unwrap().borrow_mut().next = aux_joined;
        let combine = merge_pair(self.main.clone());
        let top = combine.unwrap();
        let min = (top.borrow().key, top.borrow().id);
        let scattered = top.borrow().child.clone();
        self.main = two_pass(scattered);
        min
    }

    fn push(&mut self, key: Self::Key, value: Self::Value) -> Self::RefType {
        let new = Some(Rc::new(RefCell::new(Node {
            id: value,
            key,
            parent: None,
            child: None,
            next: self.aux.clone(),
        })));
        self.aux = new;
        self.aux.clone()
    }

    fn decrease_key(&mut self, of: Self::RefType, key: Self::Key) {
        //panics if link is empty
        let target = of.unwrap();
        if let Some(parent) = target.borrow().parent.clone() {
            parent.borrow_mut().child = None;
            target.borrow_mut().parent = None;
            target.borrow_mut().next = self.aux.clone();
            self.aux = Some(target.clone());
        }
        target.borrow_mut().key = key;
    }
}

fn merge_pair(first: Link<Node>) -> Link<Node> {
   let (a,b) = if let Some(a) = first {
        if let Some(b) = &a.borrow().next {
            (a.clone(),b.clone())
        } else {
            return Some(a.clone());
        }
    } else {
        return None;
    };
    let remainder = b.borrow().next.clone();
    a.borrow_mut().next = remainder;
    if a.borrow().key < b.borrow().key {
        let child = a.borrow().child.clone();
        b.borrow_mut().next = child;
        b.borrow_mut().parent = Some(a.clone());
        a.borrow_mut().parent = None;
        a.borrow_mut().child = Some(b.clone());
        return Some(a);
    } else {
        let child = b.borrow().child.clone();
        a.borrow_mut().next = child;
        a.borrow_mut().parent = Some(b.clone());
        b.borrow_mut().parent = None;
        b.borrow_mut().child = Some(a.clone());
        return Some(b);
    }
}

fn merge_front_to_back(start: Link<Node>) -> Link<Node> {
    let mut current = start.clone();
    loop {
        if let Some(tree) = merge_pair(current) {
            let next = tree.borrow().next.clone();
            current = Some(tree);
            if let None = next {
                return current;
            }
            continue;
        } else {
            return None;
        }
    }
}

fn merge_back_to_front(current: Link<Node>) -> Link<Node> {
    match current {
        Some(node) => {
            let next = node.borrow().next.clone();
            node.borrow_mut().next = merge_back_to_front(next);
            merge_pair(Some(node))
        }
        None => None,
    }
}
fn multipass(start: Link<Node>) -> Link<Node> {
    let mut current = start;
    let mut next_round: Link<Node> = None;
    loop {
        if let Some(tree) = merge_pair(current) {
            let remainder = tree.borrow().next.clone();
            tree.borrow_mut().next = next_round.clone();
            next_round = Some(tree);
            current = remainder;
            continue;
        } else {
            if let None = next_round {
                return None;
            }
            current = next_round;
            if let None = current.clone().unwrap().borrow().next {
                return current;
            }
            next_round = None;
        }
    }
}

fn two_pass(start: Link<Node>) -> Link<Node> {
    let mut current = start;
    let mut next_round: Link<Node> = None;
    loop {
        if let Some(tree) = merge_pair(current.clone()) {
            let remainder = tree.borrow().next.clone();
            tree.borrow_mut().next = next_round;
            next_round = Some(tree);
            current = remainder;
            continue;
        } else {
            return merge_front_to_back(next_round.clone());
        }
    }
}

#[allow(dead_code)]
fn two_pass_reverse(start: Link<Node>) -> Link<Node> {
    let mut current = start;
    let mut next_round: Link<Node> = None;
    loop {
        if let Some(tree) = merge_pair(current.clone()) {
            let remainder = tree.borrow().next.clone();
            tree.borrow_mut().next = next_round;
            next_round = Some(tree);
            current = remainder;
            continue;
        } else {
            return merge_back_to_front(next_round.clone());
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
    fn decrease_key(&mut self, of: Self::RefType, key: Self::Key);
}

pub struct Dijkstra<T: PriorityQueue> {
    pub queue: T,
    pub meta: HashMap<Vertex, (T::RefType, T::Key, T::Value), BuildHasherDefault<NoHashHasher<T::Key>>>,
}

impl<T: PriorityQueue> Dijkstra<T> {
    pub fn new(n: usize, source: Vertex) -> Self {
        let item = (T::RefType::from(source),T::Key::from(0),T::Value::from(source));
        let mut map =HashMap::with_capacity_and_hasher(n, BuildHasherDefault::default()); 
        map.insert(source, item);
        Self {
            queue: T::from(source),
            meta: map,
        }
    }
    pub fn explore(&mut self, from: T::Value, key: T::Key, e: &Neighbor) {
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

    pub fn pop_min(&mut self) -> T::Value {
        let (_, val) = self.queue.pop();
        return val;
    }
    pub fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn simple_merge() {

        let mut heap = PairingHeap::from(Vertex(1));
        heap.push(1, Vertex(2));
        heap.push(2, Vertex(3));
        heap.pop();
    }

    #[test]
    fn push_pop_pairing_heap() {
        let n = 10000;
        let mut highest_min = 0;
        let mut dijkstra: Dijkstra<PairingHeap> = Dijkstra::new(n, Vertex(1));
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
        //decrease_key
        for _ in 0..n {
            let to: Vertex = rng.gen_range(1..n).try_into().unwrap();
            let (_, key, _) = dijkstra.meta.get(&to).unwrap();
            let key = key / 2;
            dijkstra.explore(Vertex(1),0, &Neighbor { weight: key, to });
        }
        //pop
        for _ in 0..n {
            let popped = dijkstra.pop_min();
            let (_,key,_) = dijkstra.meta.remove(&popped).expect(&format!("popped {:?}", &popped));
            assert!(key >= highest_min);
            highest_min = u32::max(highest_min, key);
        }
        assert!(dijkstra.is_empty());
    }
}
