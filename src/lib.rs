#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

//! Intelligent GPU memory tiering primitives for llama.cpp backends.
//!
//! The crate models memory placement, access tracking, prediction, prefetch,
//! and profile persistence without requiring GPU hardware in tests.

/// C FFI surface sketch for the llama.cpp backend.
pub mod ffi;
/// Profile persistence.
pub mod profile;
/// Access prediction.
pub mod predictor;
/// Prefetch orchestration.
pub mod prefetch;
/// Memory tier management.
pub mod tier;
/// Access tracking.
pub mod tracker;
/// Shared public types.
pub mod types;

pub use profile::Profile;
pub use tier::TierManager;
pub use types::{AccessStats, BufferKey, TierError, TierLevel};
