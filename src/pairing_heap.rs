use std::{cell::RefCell, rc::Rc};

use crate::{dijkstra::*, dimacs::Vertex};

type Link = Option<Rc<RefCell<Box<Node>>>>;

#[derive(Debug)]
pub struct Node {
    id: Vertex,
    key: u32,
    parent: Link,
    child: Link,
    next: Link,
}

impl PartialEq for Node {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<Vertex> for Link {
    #[inline]
    fn from(value: Vertex) -> Self {
        if value == Vertex(0) {
            None
        } else {
            Some(Rc::new(RefCell::new(Box::new(Node {
                id: value,
                key: 0,
                parent: None,
                child: None,
                next: None,
            }))))
        }
    }
}

#[derive(Debug)]
pub struct PairingHeap {
    main: Link,
    aux: Link,
}

impl From<Vertex> for PairingHeap {
    #[inline]
    fn from(value: Vertex) -> Self {
        Self {
            main: Link::from(value),
            aux: None,
        }
    }
}

impl PriorityQueue for PairingHeap {
    type RefType = Link;

    type Key = u32;

    type Value = Vertex;

    #[inline]
    fn is_empty(&self) -> bool {
        if let None = self.main {
            true
        } else {
            false
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<(Self::Key, Self::Value)> {
        let aux_joined = multipass(self.aux.clone());
        self.aux = None;

        let combine = match (self.main.clone(), aux_joined.clone()) {
            (Some(main), aux) => {
                main.borrow_mut().next = aux;
                merge_pair(Some(main)).0
            }
            (None, aux) => aux,
        };

        if let Some(top) = combine {
            let scattered = top.borrow().child.clone();
            let key = top.borrow().key;
            let id = top.borrow().id;
            top.borrow_mut().child = None;

            //abandon children
            let mut curr = scattered.clone();
            while let Some(c) = curr {
                c.borrow_mut().parent = None;
                curr = c.borrow().next.clone();
            }
            //join the family
            self.main = two_pass(scattered);

            return Some((key, id));
        }
        None
    }

    #[inline]
    fn push(&mut self, key: Self::Key, value: Self::Value) -> Self::RefType {
        let new = Some(Rc::new(RefCell::new(Box::new(Node {
            id: value,
            key,
            parent: None,
            child: None,
            next: self.aux.clone(),
        }))));
        self.aux = new.clone();
        new
    }
}

impl InitDijkstra for PairingHeap {
    type Data = Search<Self>;
}

impl DecreaseKey for PairingHeap {
    fn decrease_key(&mut self, of: Self::RefType, key: Self::Key) {
        //panics if link is empty
        let target = of.unwrap();
        let parent = target.borrow().parent.clone();
        if target.borrow().id == Vertex(2868) {}
        if let Some(parent) = parent {
            let siblings = target.borrow().next.clone();
            target.borrow_mut().parent = None;
            //we all ready know this parent has children
            let first_child = parent.borrow().child.clone().unwrap();
            if first_child == target {
                parent.borrow_mut().child = siblings;
            } else {
                let mut curr_child = first_child;
                loop {
                    let next = curr_child.borrow().next.clone().unwrap();
                    if next == target {
                        curr_child.borrow_mut().next = siblings;
                        break;
                    }
                    curr_child = next.clone();
                }
            }
            target.borrow_mut().next = self.aux.clone();
            self.aux = Some(target.clone());
        }
        target.borrow_mut().key = key;
    }
}

#[allow(dead_code)]
fn find_in_link(link: Link, id: Vertex) -> bool {
    match link {
        None => false,
        Some(node) => {
            if node.clone().borrow().id == id {
                true
            } else {
                find_in_link(node.borrow().child.clone(), id)
                    || find_in_link(node.borrow().next.clone(), id)
            }
        }
    }
}

#[inline]
fn merge_pair(first: Link) -> (Link, Link) {
    let (a, b) = if let Some(a) = first {
        if let Some(b) = &a.borrow().next {
            (a.clone(), b.clone())
        } else {
            return (Some(a.clone()), None);
        }
    } else {
        return (None, None);
    };

    let remainder = b.borrow().next.clone();
    if a.borrow().key < b.borrow().key {
        let child = a.borrow().child.clone();
        b.borrow_mut().next = child;
        b.borrow_mut().parent = Some(a.clone());
        // assert_eq!(a.borrow().parent, None);
        a.borrow_mut().child = Some(b.clone());
        a.borrow_mut().next = remainder.clone();
        return (Some(a), remainder);
    } else {
        let child = b.borrow().child.clone();
        a.borrow_mut().next = child;
        a.borrow_mut().parent = Some(b.clone());
        // assert_eq!(b.borrow().parent, None);
        b.borrow_mut().child = Some(a.clone());
        return (Some(b), remainder);
    }
}

#[inline]
fn merge_front_to_back(start: Link) -> Link {
    let mut current = start.clone();
    loop {
        let (merged, remainder) = merge_pair(current);
        current = merged.clone();
        if let None = remainder {
            return current;
        }
        continue;
    }
}

fn merge_back_to_front(current: Link) -> Link {
    match current {
        Some(node) => {
            let next = node.borrow().next.clone();
            node.borrow_mut().next = merge_back_to_front(next);
            merge_pair(Some(node)).0
        }
        None => None,
    }
}

fn multipass(start: Link) -> Link {
    let mut current = start;
    let mut next_round: Link = None;
    loop {
        match merge_pair(current) {
            (Some(merged), None) => {
                if let None = next_round {
                    return Some(merged);
                }
                merged.borrow_mut().next = next_round.clone();
                return multipass(Some(merged));
            }
            (Some(merged), remainder) => {
                merged.borrow_mut().next = next_round.clone();
                next_round = Some(merged);
                current = remainder;
                continue;
            }
            (None, _) => {
                return None;
            }
        }
    }
}

#[allow(dead_code)]
#[inline]
fn two_pass(start: Link) -> Link {
    let mut current = start;
    let mut second_round: Link = None;
    loop {
        match merge_pair(current) {
            (Some(merged), None) => {
                if let None = second_round {
                    return Some(merged);
                }
                merged.borrow_mut().next = second_round.clone();
                return merge_front_to_back(Some(merged));
            }
            (Some(merged), remainder) => {
                merged.borrow_mut().next = second_round.clone();
                second_round = Some(merged);
                current = remainder;
                continue;
            }
            (None, _) => {
                return None;
            }
        }
    }
}

#[allow(dead_code)]
#[inline]
fn two_pass_reverse(start: Link) -> Link {
    let mut current = start;
    let mut second_round: Link = None;
    loop {
        match merge_pair(current) {
            (Some(merged), None) => {
                if let None = second_round {
                    return Some(merged);
                }
                merged.borrow_mut().next = second_round.clone();
                return merge_back_to_front(Some(merged));
            }
            (Some(merged), remainder) => {
                merged.borrow_mut().next = second_round.clone();
                second_round = Some(merged);
                current = remainder;
                continue;
            }
            (None, _) => {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::dijkstra::Search;

    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn simple_merge() {
        let mut heap = PairingHeap::from(Vertex(1));
        heap.push(2, Vertex(2));
        let this = heap.push(4, Vertex(3));
        assert_eq!(heap.pop(), Some((0, Vertex(1))));
        heap.decrease_key(this, 1);
        assert_eq!(heap.pop(), Some((1, Vertex(3))));
        assert_eq!(heap.pop(), Some((2, Vertex(2))));
    }

    #[test]
    fn test_leakage() {
        let mut heap = PairingHeap::from(Vertex(1));
        assert_eq!(heap.pop(), Some((0, Vertex(1))));
        let mut pushed = Vec::new();
        for i in 100..200 {
            heap.push(i * 2 + 201, Vertex(i));
            pushed.push((heap.push(i * 2, Vertex(i)), i));
        }
        for (link, i) in pushed.iter() {
            heap.decrease_key(link.clone(), *i);
            let popped = heap.pop();
            assert_eq!(popped, Some((*i, Vertex(*i))));
        }
        for i in 100..200 {
            heap.push(i * 2 + 200, Vertex(i));
            assert_eq!(heap.pop(), Some((i * 2 + 200, Vertex(i))));
            assert_eq!(heap.pop(), Some((i * 2 + 201, Vertex(i))));
        }
    }

    #[test]
    fn push_pop_pairing_heap() {
        let n = 10000;
        let mut highest_min = 0;
        let mut dijkstra: Search<PairingHeap> = Search::from((Vertex(1), n));
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
            dijkstra.explore(Vertex(1), 0, &Neighbor { weight: key, to });
        }
        //pop
        for _ in 0..n {
            let (key, popped) = dijkstra.pop_min().unwrap();
            let (_, stored_key, _) = dijkstra
                .meta
                .remove(&popped)
                .expect(&format!("popped {:?}", &popped));
            assert_eq!(key, stored_key);
            assert!(key >= highest_min);
            highest_min = u32::max(highest_min, key);
        }
        assert_eq!(None, dijkstra.pop_min());
    }
}
