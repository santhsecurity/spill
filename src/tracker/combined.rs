use crate::tracker::frequency::FrequencyTracker;
use crate::tracker::recency::RecencyTracker;
use crate::tracker::AccessTracker;
use crate::types::{BufferKey, TierError};
use std::collections::HashSet;

/// Weighted tracker combining normalized frequency and normalized recency.
#[derive(Clone, Debug)]
pub struct CombinedTracker {
    alpha: f32,
    frequency: FrequencyTracker,
    recency: RecencyTracker,
}

impl CombinedTracker {
    /// Creates a combined tracker with `alpha` in `[0.0, 1.0]`.
    ///
    /// # Errors
    ///
    /// Returns an error when `alpha` is outside the valid range or not finite.
    pub fn new(alpha: f32) -> Result<Self, TierError> {
        if !(0.0..=1.0).contains(&alpha) || !alpha.is_finite() {
            return Err(TierError::InvalidInput(
                "combined tracker alpha must be finite and within [0.0, 1.0]".to_owned(),
            ));
        }
        Ok(Self {
            alpha,
            frequency: FrequencyTracker::new(),
            recency: RecencyTracker::new(),
        })
    }

    fn max_frequency(&self) -> f32 {
        self.frequency
            .ranked()
            .first()
            .map_or(1.0, |(_key, score)| score.max(1.0))
    }

    fn max_recency(&self) -> f32 {
        self.recency
            .ranked()
            .first()
            .map_or(1.0, |(_key, score)| score.max(1.0))
    }
}

impl AccessTracker for CombinedTracker {
    fn record_access(&mut self, key: BufferKey) {
        self.frequency.record_access(key);
        self.recency.record_access(key);
    }

    fn score(&self, key: BufferKey) -> f32 {
        let freq = self.frequency.score(key) / self.max_frequency();
        let recency = self.recency.score(key) / self.max_recency();
        self.alpha.mul_add(freq, (1.0 - self.alpha) * recency)
    }

    fn ranked(&self) -> Vec<(BufferKey, f32)> {
        let mut keys = self
            .frequency
            .ranked()
            .into_iter()
            .map(|(key, _score)| key)
            .collect::<HashSet<_>>();
        keys.extend(
            self.recency
                .ranked()
                .into_iter()
                .map(|(key, _score)| key),
        );
        let mut ranked = keys
            .into_iter()
            .map(|key| (key, self.score(key)))
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
