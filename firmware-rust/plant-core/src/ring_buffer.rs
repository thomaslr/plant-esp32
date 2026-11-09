//! Fixed-capacity in-RAM history (port of `SensorRingBuffer<Capacity>`).
//!
//! Keeping history in RAM avoids flash wear. The C++ version guarded the
//! buffer with a FreeRTOS critical section; here the buffer itself is plain
//! single-threaded data and the firmware wraps it in a `Mutex`, so the
//! compiler (not discipline) guarantees nobody touches it unlocked.

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    items: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// # Panics
    /// If `capacity` is zero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be non-zero");
        Self { items: VecDeque::with_capacity(capacity), capacity }
    }

    /// Appends an item, discarding the oldest one when full.
    pub fn push(&mut self, item: T) {
        if self.items.len() == self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Up to `n` most recent items, newest first.
    pub fn recent(&self, n: usize) -> Vec<T> {
        self.items.iter().rev().take(n).cloned().collect()
    }

    /// The single most recent item, if any.
    pub fn latest(&self) -> Option<T> {
        self.items.back().cloned()
    }

    /// All items in FIFO order (oldest first).
    pub fn all(&self) -> Vec<T> {
        self.items.iter().cloned().collect()
    }

    /// Initializes ring buffer with existing items up to capacity.
    pub fn from_items(items: Vec<T>, capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be non-zero");
        let mut rb = Self::new(capacity);
        for item in items {
            rb.push(item);
        }
        rb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let rb: RingBuffer<u32> = RingBuffer::new(3);
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.latest(), None);
        assert!(rb.recent(5).is_empty());
    }

    #[test]
    fn returns_newest_first() {
        let mut rb = RingBuffer::new(5);
        for i in 1..=3 {
            rb.push(i);
        }
        assert_eq!(rb.recent(10), vec![3, 2, 1]);
        assert_eq!(rb.recent(2), vec![3, 2]);
        assert_eq!(rb.latest(), Some(3));
    }

    #[test]
    fn overwrites_oldest_when_full() {
        let mut rb = RingBuffer::new(3);
        for i in 1..=7 {
            rb.push(i);
        }
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.recent(10), vec![7, 6, 5]);
    }

    #[test]
    fn matches_cpp_capacity_100_behaviour() {
        let mut rb = RingBuffer::new(100);
        for i in 0..250u32 {
            rb.push(i);
        }
        assert_eq!(rb.len(), 100);
        let r = rb.recent(100);
        assert_eq!(r.first(), Some(&249));
        assert_eq!(r.last(), Some(&150));
    }

    #[test]
    #[should_panic]
    fn zero_capacity_is_rejected() {
        let _ = RingBuffer::<u8>::new(0);
    }
}
