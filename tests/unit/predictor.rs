use spill::predictor::locality::LocalityPredictor;
use spill::predictor::markov::MarkovPredictor;
use spill::predictor::Predictor;
use spill::BufferKey;

#[test]
fn markov_predictor_ranks_known_transition_by_frequency() {
    let mut predictor = MarkovPredictor::new();
    predictor.record_history(&[
        BufferKey(1),
        BufferKey(2),
        BufferKey(1),
        BufferKey(2),
        BufferKey(1),
        BufferKey(3),
    ]);

    let predictions = predictor.predict_next(&[BufferKey(1)]);

    assert_eq!(
        predictions[0].0,
        BufferKey(2),
        "Fix: Markov predictor must rank the most common outgoing transition first."
    );
    assert!(
        predictions[0].1 > predictions[1].1,
        "Fix: transition confidence must reflect observed transition counts."
    );
}

#[test]
fn locality_predictor_returns_last_n_keys_most_recent_first() {
    let predictor = LocalityPredictor::new(3);
    let predictions = predictor.predict_next(&[
        BufferKey(1),
        BufferKey(2),
        BufferKey(3),
        BufferKey(4),
    ]);

    assert_eq!(
        predictions
            .iter()
            .map(|(key, _confidence)| *key)
            .collect::<Vec<_>>(),
        vec![BufferKey(4), BufferKey(3), BufferKey(2)],
        "Fix: locality predictor must return the last N keys in newest-first order."
    );
}
