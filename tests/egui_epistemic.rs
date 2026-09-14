#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredBindingIdAmbiguity, EguiAuthoredContinuityStatus, EguiAuthoredDiff,
    EguiAuthoredIdAmbiguity, EguiCorrelatedDiff, WitnessDiff, assess_authored_continuity,
};

#[test]
fn duplicate_authored_identity_blocks_unique_attribution_without_guessing() {
    let diff = diff_with_ambiguity();

    let assessment = assess_authored_continuity(&diff);

    assert_eq!(assessment.status, EguiAuthoredContinuityStatus::Ambiguous);
    assert_eq!(assessment.object_ambiguity_count, 1);
    assert_eq!(assessment.binding_ambiguity_count, 1);
    assert!(assessment.unique_attribution_blocked());

    // Ambiguity is not silently converted into a guessed material change.
    assert!(diff.authored.objects_changed.is_empty());
    assert!(diff.authored.bindings_changed.is_empty());
}

#[test]
fn no_known_ambiguity_does_not_overclaim_evidence_completeness() {
    let diff = EguiCorrelatedDiff {
        before_request_id: 1,
        after_request_id: 2,
        before_pass_nr: 10,
        after_pass_nr: 11,
        semantic: empty_witness_diff(10, 11),
        authored: empty_authored_diff(),
    };

    let assessment = assess_authored_continuity(&diff);

    assert_eq!(
        assessment.status,
        EguiAuthoredContinuityStatus::NoKnownAmbiguity
    );
    assert_eq!(assessment.object_ambiguity_count, 0);
    assert_eq!(assessment.binding_ambiguity_count, 0);
    assert!(!assessment.unique_attribution_blocked());
}

fn diff_with_ambiguity() -> EguiCorrelatedDiff {
    EguiCorrelatedDiff {
        before_request_id: 7,
        after_request_id: 8,
        before_pass_nr: 9,
        after_pass_nr: 10,
        semantic: empty_witness_diff(9, 10),
        authored: EguiAuthoredDiff {
            ambiguous_ids: vec![EguiAuthoredIdAmbiguity {
                id: "canvas:duplicate".into(),
                before_count: 2,
                after_count: 1,
            }],
            binding_ambiguities: vec![EguiAuthoredBindingIdAmbiguity {
                object_id: "canvas:stable".into(),
                binding_id: "outline".into(),
                before_count: 2,
                after_count: 1,
            }],
            ..empty_authored_diff()
        },
    }
}

fn empty_authored_diff() -> EguiAuthoredDiff {
    EguiAuthoredDiff {
        objects_added: Vec::new(),
        objects_removed: Vec::new(),
        objects_changed: Vec::new(),
        bindings_added: Vec::new(),
        bindings_removed: Vec::new(),
        bindings_changed: Vec::new(),
        ambiguous_ids: Vec::new(),
        binding_ambiguities: Vec::new(),
        execution_handle_churn: Vec::new(),
    }
}

fn empty_witness_diff(before_frame: u64, after_frame: u64) -> WitnessDiff {
    WitnessDiff {
        before_frame: Some(before_frame),
        after_frame: Some(after_frame),
        version_change: None,
        viewport_change: None,
        nodes_added: Vec::new(),
        nodes_removed: Vec::new(),
        nodes_changed: Vec::new(),
        relations_added: Vec::new(),
        relations_removed: Vec::new(),
    }
}
