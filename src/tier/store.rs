use crate::types::{BufferKey, TierError};
use std::collections::HashMap;

/// Storage abstraction used by tier managers.
pub trait TierStore {
    /// Returns total byte capacity.
    fn capacity(&self) -> usize;
    /// Returns currently used bytes.
    fn used_bytes(&self) -> usize;
    /// Returns whether a key is present.
    fn contains(&self, key: BufferKey) -> bool;
    /// Inserts an owned buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the key exists or capacity is insufficient.
    fn insert(&mut self, key: BufferKey, data: Vec<u8>) -> Result<(), TierError>;
    /// Reads a buffer copy.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is absent.
    fn get(&self, key: BufferKey) -> Result<Vec<u8>, TierError>;
    /// Removes a buffer and returns it.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is absent.
    fn remove(&mut self, key: BufferKey) -> Result<Vec<u8>, TierError>;
    /// Returns all keys currently in the store.
    fn keys(&self) -> Vec<BufferKey>;
}

/// In-memory tier store for deterministic tests without GPU hardware.
#[derive(Clone, Debug)]
pub struct SimulatedStore {
    capacity: usize,
    used_bytes: usize,
    buffers: HashMap<BufferKey, Vec<u8>>,
}

impl SimulatedStore {
    /// Creates an empty simulated store.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            used_bytes: 0,
            buffers: HashMap::new(),
        }
    }

    fn available(&self) -> usize {
        self.capacity.saturating_sub(self.used_bytes)
    }
}

impl TierStore for SimulatedStore {
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn contains(&self, key: BufferKey) -> bool {
        self.buffers.contains_key(&key)
    }

    fn insert(&mut self, key: BufferKey, data: Vec<u8>) -> Result<(), TierError> {
        if self.contains(key) {
            return Err(TierError::DuplicateKey(key));
        }
        let len = data.len();
        if len > self.available() {
            return Err(TierError::CapacityExceeded {
                requested: len,
                available: self.available(),
            });
        }
        self.used_bytes += len;
        self.buffers.insert(key, data);
        Ok(())
    }

    fn get(&self, key: BufferKey) -> Result<Vec<u8>, TierError> {
        self.buffers
            .get(&key)
            .cloned()
            .ok_or(TierError::MissingKey(key))
    }

    fn remove(&mut self, key: BufferKey) -> Result<Vec<u8>, TierError> {
        let data = self
            .buffers
            .remove(&key)
            .ok_or(TierError::MissingKey(key))?;
        self.used_bytes = self.used_bytes.saturating_sub(data.len());
        Ok(data)
    }

    fn keys(&self) -> Vec<BufferKey> {
        let mut keys = self.buffers.keys().copied().collect::<Vec<_>>();
        keys.sort_unstable();
        keys
    }
}
