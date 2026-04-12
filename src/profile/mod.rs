//! Profile persistence.

use crate::types::{BufferKey, TierError, TierLevel};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const VERSION: u32 = 1;

/// Persisted workload profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    /// Hot buffers ordered by descending importance.
    pub hot_keys: Vec<BufferKey>,
    /// Persisted tier assignment per key.
    pub tier_assignment: BTreeMap<BufferKey, TierLevel>,
}

impl Profile {
    /// Creates a profile from hot keys and tier assignments.
    #[must_use]
    pub fn new(hot_keys: Vec<BufferKey>, tier_assignment: BTreeMap<BufferKey, TierLevel>) -> Self {
        Self {
            hot_keys,
            tier_assignment,
        }
    }

    /// Saves a profile as deterministic JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), TierError> {
        fs::write(path, self.to_json())
            .map_err(|error| TierError::Io(std::io::Error::new(error.kind(), error.to_string())))
    }

    /// Loads a profile saved by [`Profile::save`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the JSON is invalid.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TierError> {
        let json = fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    /// Serializes this profile to JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        let hot_keys = self
            .hot_keys
            .iter()
            .map(|key| key.0.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let assignments = self
            .tier_assignment
            .iter()
            .map(|(key, tier)| format!("{{\"key\":{},\"tier\":\"{}\"}}", key.0, tier_name(*tier)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"version\":{VERSION},\"hot_keys\":[{hot_keys}],\"tier_assignment\":[{assignments}]}}"
        )
    }

    /// Parses profile JSON produced by [`Profile::to_json`].
    ///
    /// # Errors
    ///
    /// Returns an error when mandatory fields are absent or malformed.
    pub fn from_json(json: &str) -> Result<Self, TierError> {
        let compact = json.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
        if !compact.starts_with('{') || !compact.ends_with('}') {
            return Err(TierError::Parse("profile must be a JSON object".to_owned()));
        }
        if !compact.contains("\"version\":1") {
            return Err(TierError::Parse(
                "profile version is missing or unsupported".to_owned(),
            ));
        }
        let hot_keys = parse_hot_keys(&compact)?;
        let tier_assignment = parse_assignments(&compact)?;
        Ok(Self {
            hot_keys,
            tier_assignment,
        })
    }
}

fn parse_hot_keys(json: &str) -> Result<Vec<BufferKey>, TierError> {
    let body = between(json, "\"hot_keys\":[", "]")?;
    if body.is_empty() {
        return Ok(Vec::new());
    }
    body.split(',')
        .map(|item| {
            item.parse::<u64>()
                .map(BufferKey)
                .map_err(|_| TierError::Parse("hot key must be an unsigned integer".to_owned()))
        })
        .collect()
}

fn parse_assignments(json: &str) -> Result<BTreeMap<BufferKey, TierLevel>, TierError> {
    let body = between(json, "\"tier_assignment\":[", "]")?;
    let mut assignments = BTreeMap::new();
    if body.is_empty() {
        return Ok(assignments);
    }
    for item in split_objects(body)? {
        let key_text = between(item, "\"key\":", ",\"tier\"")?;
        let tier_text = between(item, "\"tier\":\"", "\"")?;
        let key = key_text
            .parse::<u64>()
            .map(BufferKey)
            .map_err(|_| TierError::Parse("assignment key must be an unsigned integer".to_owned()))?;
        assignments.insert(key, parse_tier(tier_text)?);
    }
    Ok(assignments)
}

fn split_objects(body: &str) -> Result<Vec<&str>, TierError> {
    let mut objects = Vec::new();
    for raw in body.split("},{") {
        let item = raw.trim_start_matches('{').trim_end_matches('}');
        if item.is_empty() {
            return Err(TierError::Parse("empty tier assignment object".to_owned()));
        }
        objects.push(item);
    }
    Ok(objects)
}

fn between<'a>(text: &'a str, prefix: &str, suffix: &str) -> Result<&'a str, TierError> {
    let start = text
        .find(prefix)
        .ok_or_else(|| TierError::Parse(format!("missing JSON field prefix {prefix}")))?
        + prefix.len();
    let rest = &text[start..];
    let end = rest
        .find(suffix)
        .ok_or_else(|| TierError::Parse(format!("missing JSON field suffix {suffix}")))?;
    Ok(&rest[..end])
}

fn parse_tier(value: &str) -> Result<TierLevel, TierError> {
    match value {
        "Vram" => Ok(TierLevel::Vram),
        "Host" => Ok(TierLevel::Host),
        "Storage" => Ok(TierLevel::Storage),
        _ => Err(TierError::Parse("tier must be Vram, Host, or Storage".to_owned())),
    }
}

fn tier_name(tier: TierLevel) -> &'static str {
    match tier {
        TierLevel::Vram => "Vram",
        TierLevel::Host => "Host",
        TierLevel::Storage => "Storage",
    }
}
