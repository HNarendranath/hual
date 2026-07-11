use std::collections::HashMap;
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
    pub fn new(capacity: usize) -> Self {
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
            free: (2..capacity + 2).rev().collect(), // reverse to pop in ascending order
            map: HashMap::with_capacity(capacity),
        }
    }

    // move node at index to outward facing head of list ([dummy node 'HEAD'].next)
    fn link_first(&mut self, index: usize) {
        let old_first = self.nodes[HEAD].next;
        self.nodes[index].prev = HEAD;
        self.nodes[index].next = old_first;
        self.nodes[HEAD].next = index;
        self.nodes[old_first].prev = index;
    }

    fn unlink(&mut self, index: usize) {
        let (prev, next) = (self.nodes[index].prev, self.nodes[index].next);
        self.nodes[prev].next = next;
        self.nodes[next].prev = prev;
    }

    // move linked element to front of list
    fn touch(&mut self, index: usize) {
        self.unlink(index);
        self.link_first(index);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let index = *self.map.get(key)?;
        self.touch(index);
        self.nodes[index].value.as_ref()
    }

    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        if let Some(&index) = self.map.get(&key) {
            self.touch(index);
            return self.nodes[index].value.replace(value);
        }

        let (index, evicted) = match self.free.pop() {
            Some(index) => (index, None),
            None => {
                let to_evict = self.nodes[TAIL].prev;
                self.unlink(to_evict);
                let evicted_k = self.nodes[to_evict]
                    .key
                    .take()
                    .expect("real node always has a key");
                let evicted_v = self.nodes[to_evict].value.take();
                self.map.remove(&evicted_k);
                (to_evict, evicted_v)
            }
        };

        self.nodes[index].key = Some(key.clone());
        self.nodes[index].value = Some(value);
        self.map.insert(key, index);
        self.link_first(index);

        evicted
    }
}
