use spill::{BufferKey, TierLevel, TierManager};

#[test]
fn insert_places_buffers_in_hottest_available_tier() {
    let mut tiers = TierManager::new(4, 8, 16);
    let tier = tiers
        .insert(BufferKey(1), vec![1, 2, 3, 4])
        .expect("Fix: a buffer exactly matching VRAM capacity must fit");

    assert_eq!(
        tier,
        TierLevel::Vram,
        "Fix: insert must choose VRAM before colder tiers when capacity is available."
    );
    assert_eq!(
        tiers.get(BufferKey(1)).expect("Fix: inserted key must be readable"),
        vec![1, 2, 3, 4],
        "Fix: tier storage must preserve buffer bytes exactly."
    );
}

#[test]
fn evict_removes_buffer_from_current_tier() {
    let mut tiers = TierManager::new(4, 4, 4);
    tiers
        .insert(BufferKey(7), vec![9, 8])
        .expect("Fix: setup insert must fit");

    let removed = tiers
        .evict(BufferKey(7))
        .expect("Fix: evicting an existing key must return its bytes");

    assert_eq!(removed, vec![9, 8], "Fix: eviction must return exact bytes.");
    assert_eq!(
        tiers.tier_of(BufferKey(7)),
        None,
        "Fix: evicted keys must not remain addressable in any tier."
    );
}

#[test]
fn promote_moves_cold_buffer_to_vram_and_demotes_victim() {
    let mut tiers = TierManager::new(2, 4, 8);
    tiers
        .insert(BufferKey(1), vec![1, 1])
        .expect("Fix: first buffer must fit VRAM");
    tiers
        .insert(BufferKey(2), vec![2, 2])
        .expect("Fix: overflow buffer must fit host");

    tiers
        .promote(BufferKey(2), TierLevel::Vram)
        .expect("Fix: promotion must demote existing VRAM victim when host has room");

    assert_eq!(
        tiers.tier_of(BufferKey(2)),
        Some(TierLevel::Vram),
        "Fix: promoted key must land in VRAM."
    );
    assert_eq!(
        tiers.tier_of(BufferKey(1)),
        Some(TierLevel::Host),
        "Fix: VRAM victim must be demoted, not lost."
    );
}

#[test]
fn demote_moves_buffer_one_tier_colder() {
    let mut tiers = TierManager::new(4, 4, 4);
    tiers
        .insert(BufferKey(3), vec![3])
        .expect("Fix: setup insert must fit");

    let target = tiers
        .demote(BufferKey(3))
        .expect("Fix: VRAM buffer must demote to host");

    assert_eq!(target, TierLevel::Host, "Fix: demote must move one tier colder.");
    assert_eq!(
        tiers.tier_of(BufferKey(3)),
        Some(TierLevel::Host),
        "Fix: demoted buffer must be readable from the target tier."
    );
}

#[test]
fn capacity_overflow_errors_when_buffer_exceeds_all_tiers() {
    let mut tiers = TierManager::new(2, 2, 2);
    let error = tiers
        .insert(BufferKey(9), vec![0; 3])
        .expect_err("Fix: oversized buffers must return a typed capacity error");

    assert!(
        error.to_string().contains("Fix:"),
        "Fix: capacity errors must include an actionable repair hint."
    );
}
