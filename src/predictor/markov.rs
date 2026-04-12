use crate::predictor::Predictor;
use crate::types::BufferKey;
use std::collections::HashMap;

/// First-order Markov predictor over observed buffer transitions.
#[derive(Clone, Debug, Default)]
pub struct MarkovPredictor {
    transitions: HashMap<BufferKey, HashMap<BufferKey, u32>>,
}

impl MarkovPredictor {
    /// Creates an empty Markov predictor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one transition.
    pub fn record_transition(&mut self, from: BufferKey, to: BufferKey) {
        let destinations = self.transitions.entry(from).or_default();
        let count = destinations.entry(to).or_insert(0);
        *count = count.saturating_add(1);
    }

    /// Records all adjacent transitions in a history slice.
    pub fn record_history(&mut self, history: &[BufferKey]) {
        for window in history.windows(2) {
            if let [from, to] = window {
                self.record_transition(*from, *to);
            }
        }
    }
}

impl Predictor for MarkovPredictor {
    fn predict_next(&self, history: &[BufferKey]) -> Vec<(BufferKey, f32)> {
        let Some(last) = history.last() else {
            return Vec::new();
        };
        let Some(destinations) = self.transitions.get(last) else {
            return Vec::new();
        };
        let total = destinations.values().copied().sum::<u32>().max(1);
        let mut predictions = destinations
            .iter()
            .map(|(key, count)| {
                (
                    *key,
                    f32::from(saturating_count(*count)) / f32::from(saturating_count(total)),
                )
            })
            .collect::<Vec<_>>();
        predictions.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        predictions
    }
}

fn saturating_count(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
