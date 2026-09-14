#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiLayerOrder,
    EguiPaintKind, Rect, correlated_diff_to_agent_text, diff_correlated_captures, from_yaml,
};

#[test]
fn diff_header_preserves_material_change_and_blocks_ambiguous_attribution() {
    let before = capture(
        1,
        vec![
            object("ambiguous", "first", EguiPaintKind::Rect),
            object("ambiguous", "second", EguiPaintKind::Circle),
            object("stable", "stable", EguiPaintKind::Rect),
        ],
    );
    let after = capture(
        2,
        vec![
            object("ambiguous", "remaining", EguiPaintKind::Rect),
            object("stable", "stable", EguiPaintKind::Circle),
        ],
    );

    let diff = diff_correlated_captures(&before, &after);
    assert!(!diff.is_materially_empty());
    assert_eq!(diff.authored.ambiguous_ids.len(), 1);
    assert_eq!(diff.authored.bindings_changed.len(), 1);

    let text = correlated_diff_to_agent_text(&diff);
    let header = text.lines().next().expect("diff header");

    assert!(header.contains("materially_empty=false"));
    assert!(header.contains("authored_continuity=\"ambiguous\""));
    assert!(header.contains("unique_attribution_blocked=true"));
    assert!(header.contains("object_ambiguity_count=1"));
    assert!(header.contains("binding_ambiguity_count=0"));

    assert!(text.contains(
        "authored-binding-change object_id=\"stable\" authored_binding_id=\"part\" field=\"kind\" before=\"rect\" after=\"circle\""
    ));
    assert!(text.contains(
        "authored-ambiguity id=\"ambiguous\" before_count=2 after_count=1 matching=refused"
    ));
}

#[test]
fn diff_header_does_not_promote_no_known_ambiguity_to_completeness() {
    let before = capture(1, vec![object("stable", "stable", EguiPaintKind::Rect)]);
    let after = capture(2, vec![object("stable", "stable", EguiPaintKind::Circle)]);

    let diff = diff_correlated_captures(&before, &after);
    let text = correlated_diff_to_agent_text(&diff);
    let header = text.lines().next().expect("diff header");

    assert!(header.contains("authored_continuity=\"no_known_ambiguity\""));
    assert!(header.contains("unique_attribution_blocked=false"));
    assert!(header.contains("object_ambiguity_count=0"));
    assert!(header.contains("binding_ambiguity_count=0"));
    assert!(!header.contains("complete=true"));
    assert!(!header.contains("confidence="));
}

fn object(id: &str, role: &str, kind: EguiPaintKind) -> EguiAuthoredPaintObject {
    EguiAuthoredPaintObject {
        id: id.into(),
        role: role.into(),
        name: None,
        semantic_evidence: "intended".into(),
        bindings: vec![EguiAuthoredPaintBinding {
            authored_binding_id: Some("part".into()),
            binding_evidence: "observed".into(),
            layer_order: EguiLayerOrder::Background,
            layer_id: 42,
            shape_index: 3,
            verified_at_end_pass: true,
            kind: Some(kind),
            bounds: Some(Rect {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            }),
            clip_rect: None,
        }],
    }
}

fn capture(pass_nr: u64, authored_objects: Vec<EguiAuthoredPaintObject>) -> EguiCorrelatedCapture {
    let witness = from_yaml(&format!(
        r#"
viewwitness_version: "0.1"
capture:
  source: egui
  frame: {pass_nr}
  viewport:
    width: 100.0
    height: 50.0
    scale_factor: 1.0
nodes: []
relations: []
"#
    ))
    .expect("parse synthetic epistemic diff witness");

    EguiCorrelatedCapture {
        request_id: pass_nr,
        viewport_id: 1,
        pass_nr,
        viewport_rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        witness,
        paint: Vec::new(),
        authored_objects,
    }
}
