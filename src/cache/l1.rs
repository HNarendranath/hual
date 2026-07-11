use std::collection::HashMap;
use std::hash::Hash;

struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    prev: usize,
    next: usize,
}

// dummy nodes so all used nodes have valid prev and next
const HEAD: usize = 0;
const TAIL: usize = 1;

struct LRUCache<K, V> {
    capacity: usize,
    nodes: Vec<Node<K, V>>,
    map: HashMap<K, usize>,
    free: Vec<usize>,
}
