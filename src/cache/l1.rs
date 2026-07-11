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

impl<K: Eq + Hash + Clone, V> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "LRU capacity must be greater than 0");

        let mut nodes = Vec::with_capacity(capacity + 2);

        // empty-list state with dummy nodes
        nodes.push(Node {
            key: None,
            value: None,
            prev: TAIL,
            next: TAIL,
        });
        nodes.push(Node {
            key: None,
            value: None,
            prev: HEAD,
            next: TAIL,
        });

        // add appropriate number of empty "real" slots
        // not linked yet
        for _ in 0..capacity {
            nodes.push(Node {
                key: None,
                value: None,
                prev: HEAD,
                next: HEAD,
            })
        }

        LRUCache {
            capacity,
            nodes,
            free: (2..capacity + 2).rev().collect(),
            map: HashMap::with_capacity(capacity),
        }
    }
}
