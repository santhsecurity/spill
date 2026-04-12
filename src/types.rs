use std::{error::Error, fmt};

/// Stable identifier for a managed buffer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BufferKey(pub u64);

/// Placement tier for a managed buffer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TierLevel {
    /// Fast device memory.
    Vram,
    /// Pinned or pageable host memory.
    Host,
    /// Durable overflow storage.
    Storage,
}

impl TierLevel {
    /// Returns the next colder tier.
    #[must_use]
    pub const fn colder(self) -> Option<Self> {
        match self {
            Self::Vram => Some(Self::Host),
            Self::Host => Some(Self::Storage),
            Self::Storage => None,
        }
    }
}

/// Access counters for one buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessStats {
    /// Number of observed accesses.
    pub access_count: u64,
    /// Monotonic timestamp of the most recent access.
    pub last_access_tick: u64,
    /// Buffer size in bytes.
    pub bytes: usize,
}

impl AccessStats {
    /// Creates zeroed access statistics for a buffer size.
    #[must_use]
    pub const fn new(bytes: usize) -> Self {
        Self {
            access_count: 0,
            last_access_tick: 0,
            bytes,
        }
    }
}

/// Typed error returned by tiering, tracking, prediction, and profile code.
#[derive(Debug)]
pub enum TierError {
    /// A key was already present.
    DuplicateKey(BufferKey),
    /// A requested key was absent.
    MissingKey(BufferKey),
    /// Capacity would be exceeded.
    CapacityExceeded {
        /// Requested bytes.
        requested: usize,
        /// Available or maximum bytes.
        available: usize,
    },
    /// Input violates an API contract.
    InvalidInput(String),
    /// Filesystem operation failed.
    Io(std::io::Error),
    /// Profile data could not be parsed.
    Parse(String),
}

impl fmt::Display for TierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey(key) => write!(
                f,
                "buffer key {key:?} already exists. Fix: evict it before reinserting."
            ),
            Self::MissingKey(key) => {
                write!(f, "buffer key {key:?} is missing. Fix: insert it first.")
            }
            Self::CapacityExceeded {
                requested,
                available,
            } => write!(
                f,
                "capacity exceeded: requested {requested} bytes, available {available}. \
                 Fix: increase tier capacity or evict colder buffers."
            ),
            Self::InvalidInput(message) => write!(f, "{message}. Fix: pass a valid value."),
            Self::Io(error) => write!(f, "{error}. Fix: verify the profile path and permissions."),
            Self::Parse(message) => write!(f, "{message}. Fix: load a valid helix profile JSON."),
        }
    }
}

impl Error for TierError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TierError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
