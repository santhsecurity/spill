use spill::{BufferKey, Profile, TierLevel};
use std::collections::BTreeMap;
use std::fs;

#[test]
fn profile_save_load_roundtrips_hot_keys_and_assignments() {
    let mut assignments = BTreeMap::new();
    assignments.insert(BufferKey(1), TierLevel::Vram);
    assignments.insert(BufferKey(2), TierLevel::Host);
    let profile = Profile::new(vec![BufferKey(1), BufferKey(2)], assignments);
    let path = std::env::temp_dir().join(format!(
        "helix-profile-{}-{}.json",
        std::process::id(),
        "roundtrip"
    ));

    profile
        .save(&path)
        .expect("Fix: profile save must write deterministic JSON");
    let loaded = Profile::load(&path).expect("Fix: saved profile JSON must load");
    fs::remove_file(&path).expect("Fix: test profile file should be removable");

    assert_eq!(
        loaded, profile,
        "Fix: profile roundtrip must preserve hot key order and tier assignments exactly."
    );
}

#[test]
fn profile_rejects_corrupt_json() {
    let error = Profile::from_json("{\"version\":1,\"hot_keys\":[1],")
        .expect_err("Fix: truncated profile JSON must return an error");

    assert!(
        error.to_string().contains("Fix:"),
        "Fix: parse errors must include an actionable repair hint."
    );
}
