#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredBindingConflictKind, EguiAuthoredBindingConsistencyStatus,
    EguiAuthoredBindingResolutionStatus, EguiAuthoredPaintBinding, EguiLayerOrder, EguiPaintKind,
    assess_authored_binding_consistency, assess_authored_binding_visibility,
};

#[test]
fn m5_epistemic_contract_preserves_absence_vs_contradiction() {
    let unresolved = binding(false, None);
    let unresolved_consistency = assess_authored_binding_consistency(&unresolved);
    assert_eq!(
        unresolved_consistency.status,
        EguiAuthoredBindingConsistencyStatus::Consistent
    );
    assert!(unresolved_consistency.conflicts.is_empty());

    let unresolved_visibility = assess_authored_binding_visibility(&unresolved);
    assert_eq!(
        unresolved_visibility.resolution,
        EguiAuthoredBindingResolutionStatus::Unverified
    );
    assert_eq!(unresolved_visibility.visible_fraction, None);
    assert!(!unresolved_visibility.visibility_known());

    let contradictory = binding(false, Some(EguiPaintKind::Circle));
    let contradictory_consistency = assess_authored_binding_consistency(&contradictory);
    assert_eq!(
        contradictory_consistency.status,
        EguiAuthoredBindingConsistencyStatus::Contradictory
    );
    assert_eq!(
        contradictory_consistency.conflicts,
        vec![EguiAuthoredBindingConflictKind::UnverifiedWithKind]
    );

    let contradictory_visibility = assess_authored_binding_visibility(&contradictory);
    assert_eq!(
        contradictory_visibility.resolution,
        EguiAuthoredBindingResolutionStatus::Contradictory
    );
    assert_eq!(contradictory_visibility.visible_fraction, None);
    assert!(!contradictory_visibility.visibility_known());
}

fn binding(verified_at_end_pass: bool, kind: Option<EguiPaintKind>) -> EguiAuthoredPaintBinding {
    EguiAuthoredPaintBinding {
        authored_binding_id: Some("center".into()),
        binding_evidence: "observed".into(),
        layer_order: EguiLayerOrder::Background,
        layer_id: 42,
        shape_index: 3,
        verified_at_end_pass,
        kind,
        bounds: None,
        clip_rect: None,
    }
}
