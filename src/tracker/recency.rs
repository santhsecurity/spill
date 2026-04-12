use crate::tracker::AccessTracker;
use crate::types::BufferKey;
use std::collections::VecDeque;

/// Least-recently-used tracker backed by a deque.
#[derive(Clone, Debug, Default)]
pub struct RecencyTracker {
    order: VecDeque<BufferKey>,
}

impl RecencyTracker {
    /// Creates an empty recency tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns keys from least recent to most recent.
    #[must_use]
    pub fn lru_order(&self) -> Vec<BufferKey> {
        self.order.iter().copied().collect()
    }
}

impl AccessTracker for RecencyTracker {
    fn record_access(&mut self, key: BufferKey) {
        self.order.retain(|existing| *existing != key);
        self.order.push_back(key);
    }

    fn score(&self, key: BufferKey) -> f32 {
        self.order
            .iter()
            .position(|existing| *existing == key)
            .map_or(0.0, |index| f32::from(saturating_score(index + 1)))
    }

    fn ranked(&self) -> Vec<(BufferKey, f32)> {
        self.order
            .iter()
            .rev()
            .enumerate()
            .map(|(index, key)| (*key, f32::from(saturating_score(self.order.len() - index))))
            .collect()
    }
}

fn saturating_score(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
