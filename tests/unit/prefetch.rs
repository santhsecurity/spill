use spill::predictor::markov::MarkovPredictor;
use spill::prefetch::PrefetchController;
use spill::{BufferKey, TierLevel, TierManager};

#[test]
fn controller_promotes_predicted_host_buffer_to_vram() {
    let mut predictor = MarkovPredictor::new();
    predictor.record_transition(BufferKey(1), BufferKey(2));
    let controller =
        PrefetchController::new(predictor, 0.5, 4).expect("Fix: valid controller config");
    let mut tiers = TierManager::new(2, 4, 8);
    tiers
        .insert(BufferKey(1), vec![1, 1])
        .expect("Fix: setup VRAM insert must fit");
    tiers
        .insert(BufferKey(2), vec![2, 2])
        .expect("Fix: setup host insert must fit");

    let issued = controller
        .issue_prefetches(&[BufferKey(1)], &mut tiers)
        .expect("Fix: predicted existing key must prefetch successfully");

    assert_eq!(
        issued.len(),
        1,
        "Fix: controller must issue exactly one prefetch for the known prediction."
    );
    assert_eq!(
        issued[0].key,
        BufferKey(2),
        "Fix: issued request must reference the predicted key."
    );
    assert_eq!(
        tiers.tier_of(BufferKey(2)),
        Some(TierLevel::Vram),
        "Fix: prefetch must promote the predicted buffer into VRAM."
    );
}
