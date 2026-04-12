//! Access tracking policies.

/// Combined access tracker module.
pub mod combined;
/// Frequency access tracker module.
pub mod frequency;
/// Recency access tracker module.
pub mod recency;

use crate::types::BufferKey;

/// Tracks accesses and scores keys for retention in hotter tiers.
pub trait AccessTracker {
    /// Records one access.
    fn record_access(&mut self, key: BufferKey);
    /// Returns a score where larger values are hotter.
    fn score(&self, key: BufferKey) -> f32;
    /// Returns keys ordered from hottest to coldest.
    fn ranked(&self) -> Vec<(BufferKey, f32)>;
}
