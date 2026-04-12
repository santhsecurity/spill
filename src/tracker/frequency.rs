use crate::tracker::AccessTracker;
use crate::types::BufferKey;
use std::collections::HashMap;

/// Frequency tracker that counts accesses per key.
#[derive(Clone, Debug, Default)]
pub struct FrequencyTracker {
    counts: HashMap<BufferKey, u32>,
}

impl FrequencyTracker {
    /// Creates an empty frequency tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the raw access count for a key.
    #[must_use]
    pub fn count(&self, key: BufferKey) -> u32 {
        self.counts.get(&key).copied().unwrap_or(0)
    }
}

impl AccessTracker for FrequencyTracker {
    fn record_access(&mut self, key: BufferKey) {
        let count = self.counts.entry(key).or_insert(0);
        *count = count.saturating_add(1);
    }

    fn score(&self, key: BufferKey) -> f32 {
        f32::from(saturating_score(self.count(key)))
    }

    fn ranked(&self) -> Vec<(BufferKey, f32)> {
        let mut ranked = self
            .counts
            .iter()
            .map(|(key, count)| (*key, f32::from(saturating_score(*count))))
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        ranked
    }
}

fn saturating_score(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
