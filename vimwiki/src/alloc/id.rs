use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    ops::Range,
    sync::{Arc, Mutex, Weak},
};

lazy_static! {
    /// Represents the default range for each block of allocated ids
    static ref DEFAULT_RANGE_SIZE: Id = 10;

    /// Represents the global id allocator used by all id pools by default
    pub static ref GLOBAL_ID_ALLOCATOR: ShareableIdAllocator =
        IdAllocator::default().into_shareable();
}

/// Helper that is used purely when deserializing an id pool
#[inline]
fn global_id_allocator_downgrade() -> Weak<Mutex<IdAllocator>> {
    Arc::downgrade(&GLOBAL_ID_ALLOCATOR)
}

/// Represents the type for ids
pub type Id = usize;

/// Represents the range used for ids
type IdRange = Range<Id>;

/// Represents an id allocator that is shareable and mutable across threads
pub type ShareableIdAllocator = Arc<Mutex<IdAllocator>>;

/// Represents the allocator of ids that all pools use
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdAllocator {
    /// Represents a counter that keeps track of where an allocator is when
    /// consuming a new id
    next_id: Id,

    /// Represents size of each id range
    range_size: usize,

    /// Represents ranges of ids that have been freed from existing pools
    /// and can be reused
    freed: Vec<IdRange>,
}

impl IdAllocator {
    /// Updates the global allocator using the given instance as its starting
    /// point for internal counter and freed id ranges.
    ///
    /// Useful when restoring a serialized collection of pools and needing
    /// to maintain the state of the allocator
    pub fn using(&mut self, other: Self) {
        self.next_id = other.next_id;
        self.range_size = other.range_size;
        self.freed = other.freed;
    }

    /// Creates a new allocator using the given size for each range of
    /// ids that is allocated
    pub fn with_range_size(range_size: usize) -> Self {
        Self {
            next_id: 0,
            range_size,
            freed: Vec::new(),
        }
    }

    /// Converts this allocator into a shareable version for use with
    /// id pools
    pub fn into_shareable(self) -> ShareableIdAllocator {
        Arc::new(Mutex::new(self))
    }

    /// Returns the size of each range of ids that is allocated
    pub fn range_size(&self) -> usize {
        self.range_size
    }
}

impl Default for IdAllocator {
    /// Creates a new allocator with the default range size for ids
    fn default() -> Self {
        Self::with_range_size(*DEFAULT_RANGE_SIZE)
    }
}

impl Iterator for IdAllocator {
    type Item = IdRange;

    /// Produces a range by either yielding one of the freed pool ranges
    /// or allocating a new range of ids
    ///
    /// This should always yield a new range and should be assumed to be
    /// infinite. If the allocator has run out of ids, it will return None,
    /// and this would indicate a problem that should panic.
    fn next(&mut self) -> Option<Self::Item> {
        // If we have some range available where we do not need to allocate
        // a new range of ids, return that instead of a new allocation
        if let Some(range) = self.freed.pop() {
            Some(range)

        // If we do not have enough remaining to allocate more ids,
        // we have hit our limit and should indicate so
        } else if Id::MAX - self.next_id < self.range_size {
            None

        // Otherwise, we have nothing available to reuse and we have space
        // available for a new pool of ids, so provide that range
        } else {
            let start = self.next_id;
            self.next_id += self.range_size;
            let range = start..self.next_id;
            Some(range)
        }
    }
}

impl Extend<IdRange> for IdAllocator {
    /// Extends the collection of freed id ranges with the given iterator
    /// of id ranges; this should normally be called when a pool is dropped
    fn extend<I: IntoIterator<Item = IdRange>>(&mut self, iter: I) {
        self.freed.extend(iter);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdPool {
    /// Internally tracks the allocator used by the pool. This should normally
    /// be our global allocator, but can be swapped out. This is not serialized
    /// or deserialized and is instead updated to point to the global allocator
    /// whenever the pool is deserialized.
    #[serde(skip, default = "global_id_allocator_downgrade")]
    allocator: Weak<Mutex<IdAllocator>>,

    /// Represents the next id to return if in our available range
    next_id: Id,

    /// Represents the available ids that can be returned immediately
    /// by this pool
    available: Option<IdRange>,

    /// Represents all id ranges that have been fully consumed by the pool
    used: Vec<IdRange>,
}

impl Eq for IdPool {}
impl PartialEq for IdPool {
    fn eq(&self, other: &Self) -> bool {
        self.next_id == other.next_id
            && self.available == other.available
            && self.used == other.used
    }
}

impl IdPool {
    /// Creates a new id pool with a weak, mutable reference to the
    /// allocator to use for new ids
    pub fn new(allocator: Weak<Mutex<IdAllocator>>) -> Self {
        Self {
            allocator,
            next_id: 0,
            available: None,
            used: Vec::new(),
        }
    }

    /// Produces a clone of the weak reference to the pool's allocator
    pub fn to_allocator(&self) -> Weak<Mutex<IdAllocator>> {
        Weak::clone(&self.allocator)
    }

    /// Returns true if the pool has an id available immediately and is not
    /// dependent on allocating more ids. This does NOT indicate whether or
    /// not the allocator has more ids available to provide to the pool. In
    /// general, it should be assumed that a pool can always acquire more ids
    /// during a call to next.
    #[inline]
    pub fn has_next_available(&self) -> bool {
        self.available.is_some()
            && self.available.as_ref().unwrap().contains(&self.next_id)
    }

    /// Consumes the pool and returns a new pool whose allocator has been
    /// changed to the global allocator. Does not free up the pool's
    /// used and available id pools as they are transferred to the new pool.
    ///
    /// This is useful when restoring a pool from a serialized state and
    /// needing to reconnect it back to the global allocator
    #[inline]
    pub fn with_global_allocator(self) -> Self {
        self.with_allocator(global_id_allocator_downgrade())
    }

    /// Consumes the pool and returns a new pool whose allocator has been
    /// changed to the given weak reference. Does not free up the pool's
    /// used and available id pools as they are transferred to the new pool.
    #[inline]
    pub fn with_allocator(
        mut self,
        allocator: Weak<Mutex<IdAllocator>>,
    ) -> Self {
        Self {
            allocator,
            next_id: self.next_id,
            available: self.available.take(),
            used: self.used.drain(..).collect(),
        }
    }

    /// Merges all pools in the given iterator, returning a pool that
    /// represents them all. This will not free any of the ids used by
    /// each individual pool and will place any available ids within each
    /// pool into the used pile, meaning that there will be some additional
    /// id loss.
    ///
    /// The global id allocator will be assigned to the new pool, so this
    /// pool will need to be updated if it is using a different allocator.
    pub fn merge(id_pools: impl IntoIterator<Item = IdPool>) -> Self {
        let mut used = Vec::new();
        for mut pool in id_pools {
            used.extend(pool.used.drain(..));
            if let Some(r) = pool.available.take() {
                used.push(r);
            }
        }

        let mut pool = IdPool::default();
        pool.used = used;
        pool
    }
}

impl Default for IdPool {
    /// Creates a new pool with no range of ids yet allocated to it by
    /// the global allocator
    fn default() -> Self {
        Self::new(Arc::downgrade(&GLOBAL_ID_ALLOCATOR))
    }
}

impl Iterator for IdPool {
    type Item = Id;

    /// Pulls a new id out of the pool, potentially growing the pool if it
    /// has run out of ids
    ///
    /// This should always yield a new idand should be assumed to be
    /// infinite. If the underlying allocator has run out of ids, it will
    /// return None, and this would indicate a problem that should panic.
    fn next(&mut self) -> Option<Self::Item> {
        // If we are out of ids, check with the global allocator for a new
        // range to use and move the existing range into the used range
        if !self.has_next_available() {
            if let Some(r) = self.available.take() {
                self.used.push(r);
            }

            // If this fails by returning none, we want to short-circuit
            // return none as a result, which indicates a big problem
            if let Some(r) = self
                .allocator
                .upgrade()
                .and_then(|allocator| allocator.lock().unwrap().next())
            {
                self.next_id = r.start;
                self.available = Some(r);
            } else {
                return None;
            }
        }

        // At this point, we can safely assume that we have a valid id that
        // can be returned; we will also increment our internal tracker so
        // we can be sure not to repeat that id
        let id = self.next_id;
        self.next_id += 1;
        Some(id)
    }
}

impl Drop for IdPool {
    /// When a pool is dropped, all ids it contains are freed and given
    /// back to the allocator
    fn drop(&mut self) {
        if let Some(allocator) = self.allocator.upgrade() {
            allocator
                .lock()
                .unwrap()
                .extend(self.available.iter().chain(self.used.iter()).cloned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod id_allocator {
        use super::*;

        #[test]
        fn using_should_update_internal_state_with_provided_allocator() {
            let mut id_alloc = IdAllocator::default();
            assert_eq!(id_alloc.next_id, 0);
            assert_eq!(id_alloc.range_size, *DEFAULT_RANGE_SIZE);
            assert!(id_alloc.freed.is_empty());

            id_alloc.using(IdAllocator {
                next_id: 999,
                range_size: 123,
                freed: vec![32..35],
            });
            assert_eq!(id_alloc.next_id, 999);
            assert_eq!(id_alloc.range_size, 123);
            assert_eq!(id_alloc.freed, vec![32..35]);
        }

        #[test]
        fn next_should_return_last_range_in_freed_list_if_available() {
            let mut id_alloc = IdAllocator::default();
            id_alloc.next_id = 999;
            id_alloc.freed = vec![32..35, 38..40];

            assert_eq!(id_alloc.next(), Some(38..40));
            assert_eq!(id_alloc.freed, vec![32..35]);
        }

        #[test]
        fn next_should_return_none_if_no_freed_available_and_reached_id_limit()
        {
            let mut id_alloc = IdAllocator::default();

            // Update allocator to not have enough space for another id range
            // and to not have any ranges free
            id_alloc.next_id = Id::MAX - id_alloc.range_size() + 1;
            id_alloc.freed = vec![];

            assert_eq!(id_alloc.next(), None);
        }

        #[test]
        fn next_should_return_new_range_and_increment_internal_next_id_if_available_and_no_freed(
        ) {
            let mut id_alloc = IdAllocator::default();
            assert_eq!(id_alloc.next(), Some(0..id_alloc.range_size()));
            assert_eq!(id_alloc.next_id, id_alloc.range_size());
        }

        #[test]
        fn extend_should_add_ranges_to_freed_list() {
            let mut id_alloc = IdAllocator::default();
            id_alloc.extend(vec![5..10, 15..25]);
            id_alloc.extend(vec![2..4]);
            assert_eq!(id_alloc.freed, vec![5..10, 15..25, 2..4]);
        }

        #[test]
        fn extend_should_do_nothing_if_empty_iterator_provided() {
            let mut id_alloc = IdAllocator::default();
            id_alloc.extend(vec![]);
            id_alloc.extend(vec![5..10, 15..25]);
            id_alloc.extend(vec![]);
            assert_eq!(id_alloc.freed, vec![5..10, 15..25]);
        }
    }

    mod id_pool {
        use super::*;

        fn test_allocator_and_pool() -> (ShareableIdAllocator, IdPool) {
            let allocator = Arc::new(Mutex::new(IdAllocator::default()));
            let pool =
                IdPool::default().with_allocator(Arc::downgrade(&allocator));
            (allocator, pool)
        }

        #[test]
        fn default_should_create_a_pool_that_has_nothing_allocated_to_it() {
            let (_allocator, pool) = test_allocator_and_pool();
            assert_eq!(pool.next_id, 0);
            assert!(pool.available.is_none());
            assert!(pool.used.is_empty());
        }

        #[test]
        fn has_next_available_should_return_true_if_next_id_is_in_available_range(
        ) {
            let (_allocator, mut pool) = test_allocator_and_pool();
            pool.next_id = 0;
            pool.available = Some(0..1);

            assert!(pool.has_next_available());
        }

        #[test]
        fn has_next_available_should_return_false_if_has_no_available_range() {
            let (_allocator, mut pool) = test_allocator_and_pool();
            pool.available = None;

            assert!(!pool.has_next_available());
        }

        #[test]
        fn has_next_available_should_return_false_if_next_id_is_not_in_available_range(
        ) {
            let (_allocator, mut pool) = test_allocator_and_pool();
            pool.next_id = 0;
            pool.available = Some(1..2);

            assert!(!pool.has_next_available());
        }

        #[test]
        fn next_should_return_an_id_from_available_if_possible() {
            let (_allocator, mut pool) = test_allocator_and_pool();
            pool.next_id = 0;
            pool.available = Some(0..1);

            assert_eq!(pool.next(), Some(0));
        }

        #[test]
        fn next_should_request_a_new_range_of_ids_if_it_has_none_available() {
            let (_allocator, mut pool) = test_allocator_and_pool();
            pool.next_id = 0;
            pool.available = None;

            let maybe_id = pool.next();
            let available_start = pool
                .available
                .as_ref()
                .expect("Pool missing available")
                .start;

            assert_eq!(maybe_id, Some(available_start));
        }

        #[test]
        fn next_should_request_a_new_range_of_ids_if_it_has_run_out_of_ids() {
            let (_allocator, mut pool) = test_allocator_and_pool();
            pool.next_id = 1;
            pool.available = Some(0..1);

            let maybe_id = pool.next();
            let available_start = pool
                .available
                .as_ref()
                .expect("Pool missing available")
                .start;

            assert_eq!(maybe_id, Some(available_start));
        }

        #[test]
        fn next_should_return_none_if_it_has_no_ids_and_allocator_has_no_ids() {
            let (allocator, mut pool) = test_allocator_and_pool();

            // Force our global allocator to be maxed out
            allocator.lock().unwrap().using(IdAllocator {
                next_id: Id::MAX,
                range_size: *DEFAULT_RANGE_SIZE,
                freed: Vec::new(),
            });

            assert_eq!(pool.next(), None);
        }

        #[test]
        fn drop_should_add_available_id_range_back_into_allocator() {
            let allocator = {
                let (allocator, mut pool) = test_allocator_and_pool();
                pool.available = Some(0..1);

                allocator
            };

            assert_eq!(allocator.lock().unwrap().freed, vec![0..1]);
        }

        #[test]
        fn drop_should_add_used_id_ranges_back_into_allocator() {
            let allocator = {
                let (allocator, mut pool) = test_allocator_and_pool();
                pool.used = vec![0..1];

                allocator
            };

            assert_eq!(allocator.lock().unwrap().freed, vec![0..1]);
        }

        #[test]
        fn drop_should_add_available_and_used_id_ranges_back_into_allocator() {
            let allocator = {
                let (allocator, mut pool) = test_allocator_and_pool();
                pool.available = Some(0..1);
                pool.used = vec![1..2, 2..3];

                allocator
            };

            assert_eq!(allocator.lock().unwrap().freed, vec![0..1, 1..2, 2..3]);
        }
    }
}
