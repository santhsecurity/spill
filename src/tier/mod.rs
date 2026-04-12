//! Memory tier manager.

/// Tier storage abstractions and simulated store.
pub mod store;

use crate::tier::store::{SimulatedStore, TierStore};
use crate::types::{BufferKey, TierError, TierLevel};

/// Coordinates placement across VRAM, host memory, and storage.
#[derive(Clone, Debug)]
pub struct TierManager {
    vram: SimulatedStore,
    host: SimulatedStore,
    storage: SimulatedStore,
}

impl TierManager {
    /// Creates a tier manager backed by simulated stores.
    #[must_use]
    pub fn new(vram_bytes: usize, host_bytes: usize, storage_bytes: usize) -> Self {
        Self {
            vram: SimulatedStore::new(vram_bytes),
            host: SimulatedStore::new(host_bytes),
            storage: SimulatedStore::new(storage_bytes),
        }
    }

    /// Inserts a buffer into the hottest tier that can accept it.
    ///
    /// # Errors
    ///
    /// Returns an error if the key exists or all tiers lack capacity.
    pub fn insert(&mut self, key: BufferKey, data: Vec<u8>) -> Result<TierLevel, TierError> {
        if self.tier_of(key).is_some() {
            return Err(TierError::DuplicateKey(key));
        }
        for tier in [TierLevel::Vram, TierLevel::Host, TierLevel::Storage] {
            let available = self
                .store(tier)
                .capacity()
                .saturating_sub(self.store(tier).used_bytes());
            if data.len() <= available {
                self.store_mut(tier).insert(key, data)?;
                return Ok(tier);
            }
        }
        Err(TierError::CapacityExceeded {
            requested: data.len(),
            available: self
                .storage
                .capacity()
                .saturating_sub(self.storage.used_bytes()),
        })
    }

    /// Reads a managed buffer from any tier.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is absent.
    pub fn get(&self, key: BufferKey) -> Result<Vec<u8>, TierError> {
        let tier = self.tier_of(key).ok_or(TierError::MissingKey(key))?;
        self.store(tier).get(key)
    }

    /// Returns the current tier for a key.
    #[must_use]
    pub fn tier_of(&self, key: BufferKey) -> Option<TierLevel> {
        [TierLevel::Vram, TierLevel::Host, TierLevel::Storage]
            .into_iter()
            .find(|tier| self.store(*tier).contains(key))
    }

    /// Promotes a buffer to the requested hotter or equal tier.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is absent or the target cannot fit the buffer.
    pub fn promote(&mut self, key: BufferKey, target: TierLevel) -> Result<(), TierError> {
        let current = self.tier_of(key).ok_or(TierError::MissingKey(key))?;
        if current == target {
            return Ok(());
        }
        let data = self.store_mut(current).remove(key)?;
        if data.len() > self.store(target).capacity() {
            self.store_mut(current).insert(key, data)?;
            return Err(TierError::CapacityExceeded {
                requested: self.store(current).get(key)?.len(),
                available: self.store(target).capacity(),
            });
        }
        self.make_room(target, data.len(), Some(key))?;
        self.store_mut(target).insert(key, data)
    }

    /// Demotes a buffer by one tier.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is absent or already in storage.
    pub fn demote(&mut self, key: BufferKey) -> Result<TierLevel, TierError> {
        let current = self.tier_of(key).ok_or(TierError::MissingKey(key))?;
        let target = current.colder().ok_or_else(|| {
            TierError::InvalidInput("cannot demote a buffer already in storage".to_owned())
        })?;
        self.move_to(key, current, target)?;
        Ok(target)
    }

    /// Evicts a buffer from whichever tier currently holds it.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is absent.
    pub fn evict(&mut self, key: BufferKey) -> Result<Vec<u8>, TierError> {
        let tier = self.tier_of(key).ok_or(TierError::MissingKey(key))?;
        self.store_mut(tier).remove(key)
    }

    /// Returns used bytes in a tier.
    #[must_use]
    pub fn used_bytes(&self, tier: TierLevel) -> usize {
        self.store(tier).used_bytes()
    }

    /// Returns deterministic keys in a tier.
    #[must_use]
    pub fn keys(&self, tier: TierLevel) -> Vec<BufferKey> {
        self.store(tier).keys()
    }

    fn move_to(
        &mut self,
        key: BufferKey,
        source: TierLevel,
        target: TierLevel,
    ) -> Result<(), TierError> {
        let data = self.store_mut(source).remove(key)?;
        self.make_room(target, data.len(), Some(key))?;
        self.store_mut(target).insert(key, data)
    }

    fn make_room(
        &mut self,
        tier: TierLevel,
        bytes: usize,
        protected: Option<BufferKey>,
    ) -> Result<(), TierError> {
        if bytes > self.store(tier).capacity() {
            return Err(TierError::CapacityExceeded {
                requested: bytes,
                available: self.store(tier).capacity(),
            });
        }
        while self.store(tier).used_bytes() + bytes > self.store(tier).capacity() {
            let victim = self
                .store(tier)
                .keys()
                .into_iter()
                .find(|key| Some(*key) != protected)
                .ok_or(TierError::CapacityExceeded {
                    requested: bytes,
                    available: self.store(tier).capacity() - self.store(tier).used_bytes(),
                })?;
            if let Some(colder) = tier.colder() {
                self.move_to(victim, tier, colder)?;
            } else {
                let _removed = self.store_mut(tier).remove(victim)?;
            }
        }
        Ok(())
    }

    fn store(&self, tier: TierLevel) -> &SimulatedStore {
        match tier {
            TierLevel::Vram => &self.vram,
            TierLevel::Host => &self.host,
            TierLevel::Storage => &self.storage,
        }
    }

    fn store_mut(&mut self, tier: TierLevel) -> &mut SimulatedStore {
        match tier {
            TierLevel::Vram => &mut self.vram,
            TierLevel::Host => &mut self.host,
            TierLevel::Storage => &mut self.storage,
        }
    }
}
