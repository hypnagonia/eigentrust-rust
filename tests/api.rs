// The public API, used exactly like an external crate would.

use eigentrust::{
    eigentrust, eigentrust_with_options, EigenTrustError, EigenTrustOptions, Input, PreTrust,
    TrustEdge, TrustScores,
};

fn edges(list: &[(usize, usize, f64)]) -> Vec<TrustEdge> {
    list.iter().copied().map(TrustEdge::from).collect()
}

fn seeds(list: &[(usize, f64)]) -> Vec<PreTrust> {
    list.iter().copied().map(PreTrust::from).collect()
}

fn assert_normalized(result: &TrustScores) {
    let total: f64 = result.scores().iter().sum();
    assert!((total - 1.0).abs() < 1e-9, "scores sum to {}", total);
    assert!(result.scores().iter().all(|&s| s >= 0.0));
}

#[test]
fn basic_network() {
    let result = eigentrust(
        edges(&[(0, 1, 2.0), (1, 2, 1.0), (2, 0, 1.0), (0, 2, 1.0)]),
        seeds(&[(0, 1.0)]),
    )
    .unwrap();

    assert_eq!(result.len(), 3);
    assert!(!result.is_empty());
    assert_normalized(&result);
    assert!(result.iterations() > 0);
    assert!(result.residual() <= 1e-6 / 3.0);
    assert_eq!(result.ranking()[0].0, 0);
    assert_eq!(result.get(1), Some(result.scores()[1]));
    assert_eq!(result.get(3), None);
    assert_eq!(result.clone().into_scores(), result.scores());
}

#[test]
fn defaults() {
    let options = EigenTrustOptions::default();
    assert_eq!(options, EigenTrustOptions::new());
    assert_eq!(options.alpha(), 0.5);
    assert_eq!(options.alpha(), EigenTrustOptions::DEFAULT_ALPHA);
    assert_eq!(options.epsilon(), None);
    assert_eq!(options.max_iterations(), 10_000);

    let input = edges(&[(0, 1, 1.0), (1, 0, 3.0), (1, 2, 1.0)]);
    let a = eigentrust(input.clone(), seeds(&[(0, 1.0)])).unwrap();
    let b = eigentrust_with_options(input, seeds(&[(0, 1.0)]), &options).unwrap();
    assert_eq!(a, b);
}

#[test]
fn options_change_the_result() {
    let input = edges(&[(0, 1, 1.0), (1, 2, 1.0), (2, 1, 1.0)]);
    let low = EigenTrustOptions::default().with_alpha(0.1);
    let high = EigenTrustOptions::default().with_alpha(0.9);
    let low = eigentrust_with_options(input.clone(), seeds(&[(0, 1.0)]), &low).unwrap();
    let high = eigentrust_with_options(input, seeds(&[(0, 1.0)]), &high).unwrap();
    // more alpha keeps more trust at the seed
    assert!(high.scores()[0] > low.scores()[0]);
    assert_normalized(&low);
    assert_normalized(&high);
}

#[test]
fn custom_epsilon() {
    let input = edges(&[(0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0), (2, 1, 1.0)]);
    let loose = EigenTrustOptions::default().with_epsilon(1e-2);
    let tight = EigenTrustOptions::default().with_epsilon(1e-12);
    let loose = eigentrust_with_options(input.clone(), [], &loose).unwrap();
    let tight = eigentrust_with_options(input, [], &tight).unwrap();
    assert!(loose.iterations() < tight.iterations());
    assert!(tight.residual() <= 1e-12);
}

#[test]
fn invalid_options() {
    let input = || edges(&[(0, 1, 1.0)]);
    for alpha in [-0.1, 1.5, f64::NAN, f64::INFINITY] {
        let options = EigenTrustOptions::default().with_alpha(alpha);
        let err = eigentrust_with_options(input(), [], &options).unwrap_err();
        assert!(matches!(err, EigenTrustError::InvalidAlpha(_)), "{}", err);
    }
    for epsilon in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let options = EigenTrustOptions::default().with_epsilon(epsilon);
        let err = eigentrust_with_options(input(), [], &options).unwrap_err();
        assert!(matches!(err, EigenTrustError::InvalidEpsilon(_)), "{}", err);
    }
    let options = EigenTrustOptions::default().with_max_iterations(0);
    let err = eigentrust_with_options(input(), [], &options).unwrap_err();
    assert_eq!(err, EigenTrustError::InvalidMaxIterations);
}

#[test]
fn options_are_checked_before_input() {
    let options = EigenTrustOptions::default().with_alpha(2.0);
    let err = eigentrust_with_options([], [], &options).unwrap_err();
    assert!(matches!(err, EigenTrustError::InvalidAlpha(_)));
}

#[test]
fn invalid_weights() {
    let err = eigentrust(edges(&[(0, 1, 1.0), (1, 0, -2.0)]), []).unwrap_err();
    assert_eq!(
        err,
        EigenTrustError::NegativeWeight {
            input: Input::LocalTrust,
            position: 1,
            weight: -2.0
        }
    );

    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let err = eigentrust(edges(&[(0, 1, bad)]), []).unwrap_err();
        assert_eq!(
            err,
            EigenTrustError::NonFiniteWeight {
                input: Input::LocalTrust,
                position: 0
            }
        );
        let err = eigentrust(edges(&[(0, 1, 1.0)]), seeds(&[(0, 1.0), (1, bad)])).unwrap_err();
        assert_eq!(
            err,
            EigenTrustError::NonFiniteWeight {
                input: Input::PreTrust,
                position: 1
            }
        );
    }

    let err = eigentrust(edges(&[(0, 1, f64::MAX), (0, 2, f64::MAX)]), []).unwrap_err();
    assert_eq!(err, EigenTrustError::WeightOverflow(Input::LocalTrust));
}

#[test]
fn invalid_peers() {
    let err = eigentrust(edges(&[(0, usize::MAX, 1.0)]), []).unwrap_err();
    assert_eq!(
        err,
        EigenTrustError::InvalidPeer {
            input: Input::LocalTrust,
            position: 0
        }
    );
    assert_eq!(
        eigentrust([], []).unwrap_err(),
        EigenTrustError::EmptyNetwork
    );
}

#[test]
fn duplicate_edges_keep_the_last_weight() {
    let dup = eigentrust(
        edges(&[
            (0, 1, 5.0),
            (0, 2, 1.0),
            (0, 1, 1.0),
            (1, 0, 1.0),
            (2, 0, 1.0),
        ]),
        seeds(&[(0, 1.0)]),
    )
    .unwrap();
    let clean = eigentrust(
        edges(&[(0, 2, 1.0), (0, 1, 1.0), (1, 0, 1.0), (2, 0, 1.0)]),
        seeds(&[(0, 1.0)]),
    )
    .unwrap();
    assert_eq!(dup.scores(), clean.scores());
    assert_normalized(&dup);
}

#[test]
fn duplicate_pre_trust_keeps_the_last_weight() {
    let input = edges(&[(0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0), (2, 3, 1.0)]);
    let dup = eigentrust(
        input.clone(),
        seeds(&[(0, 1.0), (3, 1.0), (0, 5.0), (0, 2.0)]),
    )
    .unwrap();
    let clean = eigentrust(input, seeds(&[(0, 2.0), (3, 1.0)])).unwrap();
    assert_eq!(dup.scores(), clean.scores());
    assert_normalized(&dup);
}

#[test]
fn zero_and_empty_pre_trust_are_uniform() {
    let input = edges(&[(0, 1, 1.0), (1, 2, 1.0), (2, 0, 2.0), (2, 1, 1.0)]);
    let empty = eigentrust(input.clone(), []).unwrap();
    let zero = eigentrust(input.clone(), seeds(&[(1, 0.0)])).unwrap();
    let uniform = eigentrust(input, seeds(&[(0, 1.0), (1, 1.0), (2, 1.0)])).unwrap();
    assert_eq!(empty.scores(), zero.scores());
    assert_eq!(empty.scores(), uniform.scores());
}

#[test]
fn weights_are_relative_per_truster() {
    let a = eigentrust(edges(&[(0, 1, 1.0), (0, 2, 3.0), (1, 0, 1.0)]), []).unwrap();
    let b = eigentrust(edges(&[(0, 1, 10.0), (0, 2, 30.0), (1, 0, 7.0)]), []).unwrap();
    assert_eq!(a.scores(), b.scores());
}

#[test]
fn zero_weight_means_no_trust() {
    let with_zero = eigentrust(edges(&[(0, 1, 1.0), (0, 2, 0.0), (1, 0, 1.0)]), []).unwrap();
    let without = eigentrust(edges(&[(0, 1, 1.0), (1, 0, 1.0), (2, 2, 0.0)]), []).unwrap();
    assert_eq!(with_zero.scores(), without.scores());
}

#[test]
fn every_index_below_the_maximum_is_a_peer() {
    // peer 1 appears nowhere but still gets a score
    let result = eigentrust(edges(&[(0, 2, 1.0), (2, 0, 1.0)]), []).unwrap();
    assert_eq!(result.len(), 3);
    assert_normalized(&result);
    // pre-trust alone can extend the network
    let result = eigentrust(edges(&[(0, 1, 1.0)]), seeds(&[(4, 1.0)])).unwrap();
    assert_eq!(result.len(), 5);
    assert_normalized(&result);
}

#[test]
fn sybil_cluster_gets_little_trust() {
    // honest ring 0..4 seeded at 0; sybils 5..9 vouch only for each other and get one
    // link from peer 4
    let mut input = Vec::new();
    for i in 0..5 {
        input.push(TrustEdge::new(i, (i + 1) % 5, 1.0));
    }
    input.push(TrustEdge::new(4, 5, 0.1));
    for a in 5..10 {
        for b in 5..10 {
            if a != b {
                input.push(TrustEdge::new(a, b, 10.0));
            }
        }
    }
    let options = EigenTrustOptions::default().with_alpha(0.5);
    let result = eigentrust_with_options(input, [PreTrust::new(0, 1.0)], &options).unwrap();
    let sybil_share: f64 = result.scores()[5..].iter().sum();
    assert!(sybil_share < 0.05, "sybils hold {}", sybil_share);
}

#[test]
fn alpha_zero_on_a_periodic_network_does_not_converge() {
    let options = EigenTrustOptions::default()
        .with_alpha(0.0)
        .with_max_iterations(100);
    let err = eigentrust_with_options(
        edges(&[(0, 1, 1.0), (1, 0, 1.0)]),
        seeds(&[(0, 1.0)]),
        &options,
    )
    .unwrap_err();
    match err {
        EigenTrustError::NotConverged {
            iterations,
            residual,
        } => {
            assert_eq!(iterations, 100);
            assert!(residual > 1.0);
        }
        other => panic!("unexpected {}", other),
    }
    // with any teleport it converges
    let options = EigenTrustOptions::default().with_alpha(0.05);
    let result = eigentrust_with_options(
        edges(&[(0, 1, 1.0), (1, 0, 1.0)]),
        seeds(&[(0, 1.0)]),
        &options,
    )
    .unwrap();
    assert_normalized(&result);
}

#[test]
fn deterministic() {
    let input: Vec<TrustEdge> = (0..2_000)
        .map(|i| TrustEdge::new(i % 997, (i * 7 + 3) % 997, (i % 5 + 1) as f64))
        .collect();
    let a = eigentrust(input.clone(), [PreTrust::new(0, 1.0)]).unwrap();
    let b = eigentrust(input, [PreTrust::new(0, 1.0)]).unwrap();
    assert_eq!(a, b);
    assert_normalized(&a);
}

#[test]
fn errors_are_std_errors_with_messages() {
    fn as_std(e: EigenTrustError) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(e)
    }
    let messages = [
        EigenTrustError::InvalidAlpha(2.0),
        EigenTrustError::EmptyNetwork,
        EigenTrustError::NegativeWeight {
            input: Input::PreTrust,
            position: 3,
            weight: -1.0,
        },
    ]
    .map(|e| as_std(e).to_string());
    assert_eq!(messages[0], "alpha must be in [0, 1], got 2");
    assert_eq!(messages[1], "the network has no peers");
    assert_eq!(
        messages[2],
        "pre-trust item 3: weight -1 is negative; distrust is not supported"
    );
}

#[test]
fn tuples_convert_into_inputs() {
    assert_eq!(TrustEdge::from((1, 2, 0.5)), TrustEdge::new(1, 2, 0.5));
    assert_eq!(PreTrust::from((4, 0.5)), PreTrust::new(4, 0.5));
    let result = eigentrust(
        [(0, 1, 1.0), (1, 0, 1.0)].map(TrustEdge::from),
        [(0, 1.0)].map(PreTrust::from),
    )
    .unwrap();
    assert_normalized(&result);
}
