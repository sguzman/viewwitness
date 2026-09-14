#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredBindingConflictKind, EguiAuthoredBindingConsistencyStatus,
    EguiAuthoredBindingIdAmbiguity, EguiAuthoredBindingResolutionStatus,
    EguiAuthoredContinuityStatus, EguiAuthoredDiff, EguiAuthoredIdAmbiguity,
    EguiAuthoredPaintBinding, EguiCorrelatedDiff, EguiLayerOrder, EguiPaintKind, Rect, WitnessDiff,
    assess_authored_binding_consistency, assess_authored_binding_visibility,
    assess_authored_continuity,
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

#[test]
fn unresolved_binding_visibility_is_unknown_not_zero() {
    let unresolved = authored_binding(false, None, None, None);

    // The low-level geometry helper has historically returned zero when there
    // is no resolved geometry. The epistemic assessment must not promote that
    // mechanical value into an observation that the binding is invisible.
    assert_eq!(unresolved.visible_fraction(), 0.0);

    let assessment = assess_authored_binding_visibility(&unresolved);
    assert_eq!(
        assessment.resolution,
        EguiAuthoredBindingResolutionStatus::Unverified
    );
    assert_eq!(assessment.visible_fraction, None);
    assert!(!assessment.visibility_known());
}

#[test]
fn verified_zero_visibility_remains_known_zero() {
    let verified_clipped = authored_binding(
        true,
        Some(EguiPaintKind::Circle),
        Some(Rect {
            x: 20.0,
            y: 20.0,
            width: 8.0,
            height: 8.0,
        }),
        Some(Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }),
    );

    assert_eq!(verified_clipped.visible_fraction(), 0.0);

    let assessment = assess_authored_binding_visibility(&verified_clipped);
    assert_eq!(
        assessment.resolution,
        EguiAuthoredBindingResolutionStatus::Verified
    );
    assert_eq!(assessment.visible_fraction, Some(0.0));
    assert!(assessment.visibility_known());
}

#[test]
fn verified_slot_without_usable_bounds_keeps_visibility_unknown() {
    let verified_without_bounds = authored_binding(true, Some(EguiPaintKind::Noop), None, None);

    assert_eq!(verified_without_bounds.visible_fraction(), 0.0);

    let consistency = assess_authored_binding_consistency(&verified_without_bounds);
    assert_eq!(
        consistency.status,
        EguiAuthoredBindingConsistencyStatus::Consistent
    );
    assert!(consistency.conflicts.is_empty());

    let visibility = assess_authored_binding_visibility(&verified_without_bounds);
    assert_eq!(
        visibility.resolution,
        EguiAuthoredBindingResolutionStatus::Verified
    );
    assert_eq!(visibility.visible_fraction, None);
    assert!(!visibility.visibility_known());
}

#[test]
fn verified_without_kind_is_explicit_contradiction_and_blocks_visibility_claim() {
    let contradictory = authored_binding(
        true,
        None,
        Some(Rect {
            x: 10.0,
            y: 10.0,
            width: 8.0,
            height: 8.0,
        }),
        None,
    );

    let consistency = assess_authored_binding_consistency(&contradictory);
    assert_eq!(
        consistency.status,
        EguiAuthoredBindingConsistencyStatus::Contradictory
    );
    assert_eq!(
        consistency.conflicts,
        vec![EguiAuthoredBindingConflictKind::VerifiedWithoutKind]
    );
    assert!(consistency.contradictory());

    let visibility = assess_authored_binding_visibility(&contradictory);
    assert_eq!(
        visibility.resolution,
        EguiAuthoredBindingResolutionStatus::Contradictory
    );
    assert_eq!(visibility.visible_fraction, None);
    assert!(!visibility.visibility_known());
}

#[test]
fn unverified_slot_with_final_paint_testimony_preserves_all_conflicts() {
    let contradictory = authored_binding(
        false,
        Some(EguiPaintKind::Circle),
        Some(Rect {
            x: 10.0,
            y: 10.0,
            width: 8.0,
            height: 8.0,
        }),
        Some(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }),
    );

    let consistency = assess_authored_binding_consistency(&contradictory);
    assert_eq!(
        consistency.status,
        EguiAuthoredBindingConsistencyStatus::Contradictory
    );
    assert_eq!(
        consistency.conflicts,
        vec![
            EguiAuthoredBindingConflictKind::UnverifiedWithKind,
            EguiAuthoredBindingConflictKind::UnverifiedWithBounds,
            EguiAuthoredBindingConflictKind::UnverifiedWithClipRect,
        ]
    );

    let visibility = assess_authored_binding_visibility(&contradictory);
    assert_eq!(
        visibility.resolution,
        EguiAuthoredBindingResolutionStatus::Contradictory
    );
    assert_eq!(visibility.visible_fraction, None);
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

fn authored_binding(
    verified_at_end_pass: bool,
    kind: Option<EguiPaintKind>,
    bounds: Option<Rect>,
    clip_rect: Option<Rect>,
) -> EguiAuthoredPaintBinding {
    EguiAuthoredPaintBinding {
        authored_binding_id: Some("center".into()),
        binding_evidence: "observed".into(),
        layer_order: EguiLayerOrder::Background,
        layer_id: 42,
        shape_index: 3,
        verified_at_end_pass,
        kind,
        bounds,
        clip_rect,
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
