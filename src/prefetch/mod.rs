//! Prefetch orchestration.

use crate::predictor::Predictor;
use crate::tier::TierManager;
use crate::types::{BufferKey, TierError, TierLevel};

/// Prefetch decision and confidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrefetchRequest {
    /// Buffer requested for promotion.
    pub key: BufferKey,
    /// Predictor confidence.
    pub confidence: f32,
}

/// Uses a predictor to promote likely future buffers into VRAM.
#[derive(Clone, Debug)]
pub struct PrefetchController<P> {
    predictor: P,
    min_confidence: f32,
    max_requests: usize,
}

impl<P: Predictor> PrefetchController<P> {
    /// Creates a controller.
    ///
    /// # Errors
    ///
    /// Returns an error if confidence is outside `[0.0, 1.0]` or not finite.
    pub fn new(predictor: P, min_confidence: f32, max_requests: usize) -> Result<Self, TierError> {
        if !(0.0..=1.0).contains(&min_confidence) || !min_confidence.is_finite() {
            return Err(TierError::InvalidInput(
                "prefetch confidence must be finite and within [0.0, 1.0]".to_owned(),
            ));
        }
        Ok(Self {
            predictor,
            min_confidence,
            max_requests,
        })
    }

    /// Issues promotions for predicted keys that meet the confidence threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if a predicted key is absent or cannot be promoted.
    pub fn issue_prefetches(
        &self,
        history: &[BufferKey],
        tiers: &mut TierManager,
    ) -> Result<Vec<PrefetchRequest>, TierError> {
        let mut issued = Vec::new();
        for (key, confidence) in self.predictor.predict_next(history) {
            if issued.len() == self.max_requests {
                break;
            }
            if confidence < self.min_confidence {
                continue;
            }
            match tiers.tier_of(key) {
                Some(TierLevel::Vram) => {}
                Some(_) => {
                    tiers.promote(key, TierLevel::Vram)?;
                    issued.push(PrefetchRequest { key, confidence });
                }
                None => return Err(TierError::MissingKey(key)),
            }
        }
        Ok(issued)
    }
}
