use std::fs;
use std::path::Path;

#[test]
fn independent_handle_verifier_contains_no_repair_recipe_or_source_mutation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let verifier = fs::read_to_string(root.join("scripts/m5-verify-handle-candidate.sh"))
        .expect("read independent M5 verifier");

    for forbidden in [
        "examples/showcase.rs",
        "vec2(60.0",
        "vec2(0.0",
        "git checkout",
        "git restore",
        "git apply",
        "sed -i",
        "write_text(",
        ".replace(",
    ] {
        assert!(
            !verifier.contains(forbidden),
            "independent verifier must not contain repair/source-mutation recipe {forbidden:?}"
        );
    }

    assert!(verifier.contains("capture-exact --yaml"));
    assert!(verifier.contains("diff-exact"));
    assert!(verifier.contains("--object=showcase:painted-rectangle --binding=handle"));
    assert!(verifier.contains("candidate moved rectangle outline"));
    assert!(verifier.contains("candidate handle remains horizontally disconnected"));
}

#[test]
fn reference_patcher_is_explicitly_separate_from_independent_verifier() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let patcher = fs::read_to_string(root.join("scripts/m5-reference-handle-patcher.sh"))
        .expect("read reference M5 patcher");
    let verifier = fs::read_to_string(root.join("scripts/m5-verify-handle-candidate.sh"))
        .expect("read independent M5 verifier");

    assert!(patcher.contains("vec2(60.0"));
    assert!(patcher.contains("vec2(0.0"));
    assert!(!verifier.contains("vec2(60.0"));
    assert!(!verifier.contains("vec2(0.0"));
}
