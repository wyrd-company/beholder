// ---
// relationships:
//   validates: stored-results
// ---
//! Persisted phase-one records captured with sha2 0.10 and tree-sitter 0.26.
//! Compare whole records, including identities, fingerprints, imports and references.
#[test]
fn phase_one_records_match_the_previous_dependency_outputs() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/phase-one-compatibility.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let path = case["path"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let actual = beholder::phase1::analyze(path, source);
        if actual.language.is_some() {
            assert!(
                !actual.symbols.is_empty(),
                "fixture must exercise symbols: {path}"
            );
        }
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            case["analysis"],
            "{path}"
        );
    }
}
