use super::*;

#[test]
#[should_panic(expected = "capacity must be greater than 0")]
fn new_with_zero_capacity_panics() {
    let _cache: LRUCache<i32, &str> = LRUCache::new(0);
}

#[test]
fn get_on_empty_cache_returns_none() {
    let mut cache: LRUCache<i32, &str> = LRUCache::new(2);
    assert_eq!(cache.get(&1), None);
}

#[test]
fn put_new_key_under_capacity_returns_none() {
    let mut cache = LRUCache::new(2);
    assert_eq!(cache.put(1, "a"), None);
    assert_eq!(cache.put(2, "b"), None);
}

#[test]
fn get_returns_value_that_was_put() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    assert_eq!(cache.get(&1), Some(&"a"));
}

#[test]
fn get_on_missing_key_returns_none_when_other_keys_present() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    assert_eq!(cache.get(&2), None);
}

#[test]
fn put_on_existing_key_returns_old_value_and_updates_it() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");

    let replaced = cache.put(1, "z");

    assert_eq!(replaced, Some("a"));
    assert_eq!(cache.get(&1), Some(&"z"));
}

#[test]
fn put_on_existing_key_does_not_evict_other_entries() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    cache.put(2, "b");

    cache.put(1, "z");

    assert_eq!(cache.get(&1), Some(&"z"));
    assert_eq!(cache.get(&2), Some(&"b"));
}

#[test]
fn put_on_existing_key_promotes_recency() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    cache.put(2, "b");

    // updating 1 should promote it to MRU, leaving 2 as the LRU entry
    cache.put(1, "z");

    let evicted = cache.put(3, "c");

    assert_eq!(evicted, Some("b"));
    assert_eq!(cache.get(&1), Some(&"z"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn put_at_capacity_evicts_least_recently_used() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    cache.put(2, "b");

    let evicted = cache.put(3, "c");

    assert_eq!(evicted, Some("a"));
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some(&"b"));
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn get_promotes_key_so_it_survives_the_next_eviction() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    cache.put(2, "b");

    // touch 1, making 2 the least-recently-used entry
    assert_eq!(cache.get(&1), Some(&"a"));

    let evicted = cache.put(3, "c");

    assert_eq!(evicted, Some("b"));
    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn eviction_order_follows_multiple_touches() {
    let mut cache = LRUCache::new(3);
    cache.put(1, "a");
    cache.put(2, "b");
    cache.put(3, "c");

    // recency order (MRU -> LRU) is now [3, 2, 1]; touching 1 then 2 makes it [2, 1, 3]
    cache.get(&1);
    cache.get(&2);

    let evicted = cache.put(4, "d");

    assert_eq!(evicted, Some("c"));
    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), Some(&"b"));
    assert_eq!(cache.get(&3), None);
    assert_eq!(cache.get(&4), Some(&"d"));
}

#[test]
fn capacity_one_evicts_previous_entry_on_every_new_put() {
    let mut cache = LRUCache::new(1);
    cache.put(1, "a");

    let evicted = cache.put(2, "b");

    assert_eq!(evicted, Some("a"));
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some(&"b"));
}

#[test]
fn evicted_slot_can_be_reused_without_stale_state() {
    let mut cache = LRUCache::new(2);
    cache.put(1, "a");
    cache.put(2, "b");

    // evicts key 1 ("a"), physically reclaiming its slot
    let evicted_first = cache.put(3, "c");
    assert_eq!(evicted_first, Some("a"));
    assert_eq!(cache.get(&1), None);

    // reinserts key 1 as a brand-new entry into a reclaimed slot; must not
    // resurrect the old ("a") value or leak stale links from its previous life
    let evicted_second = cache.put(1, "z");
    assert_eq!(evicted_second, Some("b"));
    assert_eq!(cache.get(&1), Some(&"z"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn repeated_evictions_leave_only_the_most_recent_capacity_keys() {
    let mut cache = LRUCache::new(2);
    for i in 1..=5 {
        cache.put(i, i * 10);
    }

    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), None);
    assert_eq!(cache.get(&4), Some(&40));
    assert_eq!(cache.get(&5), Some(&50));
}

#[test]
fn works_with_owned_non_copy_keys_and_values() {
    let mut cache: LRUCache<String, String> = LRUCache::new(2);
    cache.put("a".to_string(), "alpha".to_string());
    cache.put("b".to_string(), "beta".to_string());

    // touch "a", making "b" the least-recently-used entry
    assert_eq!(cache.get(&"a".to_string()), Some(&"alpha".to_string()));

    let evicted = cache.put("c".to_string(), "gamma".to_string());

    assert_eq!(evicted, Some("beta".to_string()));
    assert_eq!(cache.get(&"b".to_string()), None);
    assert_eq!(cache.get(&"a".to_string()), Some(&"alpha".to_string()));
    assert_eq!(cache.get(&"c".to_string()), Some(&"gamma".to_string()));
}
