// eigentrust::csv, used like an external crate would. Needs the `csv` feature.
#![cfg(feature = "csv")]

use eigentrust::csv::{ErrorKind, Network};
use eigentrust::{EigenTrustError, EigenTrustOptions, Input};

fn rank(lt: &str, pt: &str, alpha: f64) -> Vec<(String, f64)> {
    let network = Network::from_csv(lt, pt).unwrap();
    let options = EigenTrustOptions::default().with_alpha(alpha);
    let result = network.eigentrust(&options).unwrap();
    network
        .ranking(&result)
        .into_iter()
        .filter(|&(_, s)| s > 0.0)
        .map(|(n, s)| (n.to_string(), s))
        .collect()
}

// Reference values carried over from the original implementation.
#[test]
fn reference_results() {
    let lt =
        "i,j,v\nalice,bob,11.31571\n2,3,269916.08616\n4,5,3173339.366896588\n6,5,46589750.00759474";
    let pt = "i,v\nalice,0.14285714285714285\nbob,0.14285714285714285\n2,0.14285714285714285\n3,0.14285714285714285\n4,0.14285714285714285\n5,0.14285714285714285\n6,0.14285714285714285";
    let r = rank(lt, pt, 0.5);
    assert_eq!(r.len(), 7);
    assert_eq!(r[0], ("5".to_string(), 0.22222219873601323));
    assert_eq!(r[1].0, "bob");

    let r = rank(
        "alice,bob,11.31571\n2,3,269916.08616\n4,5,3173339.366896588\n6,5,46589750.00759474",
        "alice,1",
        0.5,
    );
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], ("alice".to_string(), 0.6666666865348816));
}

#[test]
fn reference_file() {
    let r = rank(
        include_str!("../example/localtrust2.csv"),
        include_str!("../example/pretrust2.csv"),
        0.5,
    );
    assert_eq!(r.len(), 9);
    // the file contains duplicate (i, j) records, the last one wins
    assert_eq!(
        r[0],
        (
            "0x84e1056ed1b76fb03b43e924ef98833dba394b2b".to_string(),
            0.40356129084997394
        )
    );
    assert_eq!(r[1].0, "0x9fc3b33884e1d056a8ca979833d686abd267f9f8");
}

#[test]
fn peers_are_indexed_by_first_appearance() {
    let network = Network::from_csv("b,a\nc,b,2\na,c", "b").unwrap();
    assert_eq!(network.peers(), ["b", "a", "c"]);
    assert_eq!(network.local_trust().len(), 3);
    assert_eq!(network.local_trust()[1].weight, 2.0);
    assert_eq!(network.pre_trust()[0].peer, 0);
}

#[test]
fn quoted_names() {
    let r = rank(
        "\"Smith, J\",alice\nalice,\"Smith, J\",2",
        "\"Smith, J\"",
        0.5,
    );
    assert_eq!(r[0].0, "Smith, J");
}

#[test]
fn errors_have_input_and_line() {
    let err = Network::from_csv("from,to,weight\na,b,1\n\nb,c,NaN", "a").unwrap_err();
    assert_eq!(err.input(), Input::LocalTrust);
    assert_eq!(err.line(), 4);
    assert_eq!(err.kind(), &ErrorKind::NonFiniteWeight("NaN".to_string()));
    assert_eq!(
        err.to_string(),
        "local trust CSV, line 4: weight \"NaN\" must be finite"
    );

    let err = Network::from_csv("a,b,1\nb,c,-2", "a").unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NegativeWeight(_)));

    let err = Network::from_csv("a,b\n", "a\nzed").unwrap_err();
    assert_eq!(err.input(), Input::PreTrust);
    assert_eq!(err.line(), 2);
    assert_eq!(err.kind(), &ErrorKind::UnknownPeer("zed".to_string()));

    let err = Network::from_csv("a,b,1\njust-one-field", "").unwrap_err();
    assert_eq!(err.kind(), &ErrorKind::TooFewFields);

    let err = Network::from_csv("a,b,1\na,c,x", "").unwrap_err();
    assert_eq!(err.kind(), &ErrorKind::InvalidWeight("x".to_string()));
}

#[test]
fn empty_input_is_an_empty_network() {
    let network = Network::from_csv("", "").unwrap();
    let err = network
        .eigentrust(&EigenTrustOptions::default())
        .unwrap_err();
    assert_eq!(err, EigenTrustError::EmptyNetwork);
}
