use crate::predictor::Predictor;
use crate::types::BufferKey;

/// Locality predictor that returns the last `window` accessed keys.
#[derive(Clone, Debug)]
pub struct LocalityPredictor {
    window: usize,
}

impl LocalityPredictor {
    /// Creates a predictor that returns at most `window` recent keys.
    #[must_use]
    pub const fn new(window: usize) -> Self {
        Self { window }
    }
}

impl Predictor for LocalityPredictor {
    fn predict_next(&self, history: &[BufferKey]) -> Vec<(BufferKey, f32)> {
        if self.window == 0 || history.is_empty() {
            return Vec::new();
        }
        let start = history.len().saturating_sub(self.window);
        let recent = &history[start..];
        recent
            .iter()
            .rev()
            .enumerate()
            .map(|(index, key)| {
                let denominator = self.window.max(1);
                let confidence = f32::from(saturating_count((denominator - index).max(1)))
                    / f32::from(saturating_count(denominator));
                (*key, confidence)
            })
            .collect()
    }
}

fn saturating_count(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
