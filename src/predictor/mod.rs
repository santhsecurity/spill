//! Access predictors.

/// Locality predictor module.
pub mod locality;
/// Markov predictor module.
pub mod markov;

use crate::types::BufferKey;

/// Predicts future buffer accesses from observed history.
pub trait Predictor {
    /// Returns predicted next keys with confidence in `[0.0, 1.0]`.
    fn predict_next(&self, history: &[BufferKey]) -> Vec<(BufferKey, f32)>;
}
