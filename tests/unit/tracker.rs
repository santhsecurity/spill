use spill::tracker::combined::CombinedTracker;
use spill::tracker::frequency::FrequencyTracker;
use spill::tracker::recency::RecencyTracker;
use spill::tracker::AccessTracker;
use spill::BufferKey;

#[test]
fn frequency_tracker_counts_repeated_accesses() {
    let mut tracker = FrequencyTracker::new();
    tracker.record_access(BufferKey(1));
    tracker.record_access(BufferKey(1));
    tracker.record_access(BufferKey(2));

    assert_eq!(
        tracker.count(BufferKey(1)),
        2,
        "Fix: repeated accesses to the same key must increment its frequency."
    );
    assert_eq!(
        tracker.ranked()[0].0,
        BufferKey(1),
        "Fix: frequency ranking must place the most-accessed key first."
    );
}

#[test]
fn recency_tracker_moves_reaccessed_key_to_most_recent() {
    let mut tracker = RecencyTracker::new();
    tracker.record_access(BufferKey(1));
    tracker.record_access(BufferKey(2));
    tracker.record_access(BufferKey(1));

    assert_eq!(
        tracker.lru_order(),
        vec![BufferKey(2), BufferKey(1)],
        "Fix: LRU order must remove stale duplicates before appending reaccessed keys."
    );
    assert!(
        tracker.score(BufferKey(1)) > tracker.score(BufferKey(2)),
        "Fix: most recent keys must receive higher recency scores."
    );
}

#[test]
fn combined_tracker_balances_frequency_and_recency() {
    let mut tracker =
        CombinedTracker::new(0.75).expect("Fix: alpha inside range must construct tracker");
    for _ in 0..4 {
        tracker.record_access(BufferKey(1));
    }
    tracker.record_access(BufferKey(2));

    assert_eq!(
        tracker.ranked()[0].0,
        BufferKey(1),
        "Fix: with alpha weighted toward frequency, repeated hot keys must outrank one recent key."
    );
}
