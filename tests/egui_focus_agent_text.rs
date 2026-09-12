#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiLayerOrder,
    EguiPaintKind, EguiPaintObservation, Rect, correlated_capture_authored_focus_to_agent_text,
    from_yaml,
};

#[test]
fn authored_focus_selects_one_binding_and_labels_omissions() {
    let capture = sample_capture();
    let text =
        correlated_capture_authored_focus_to_agent_text(&capture, "canvas:node", Some("handle"));

    assert!(text.starts_with(
        "egui-correlated-focus request=7 viewport_id=2 pass=9 viewport_rect=[0,0,100,50] object_id=\"canvas:node\" binding_id=\"handle\" object_match_count=1 binding_match_count=1 correlation=same_full_output projection=authored_focus omitted=canonical_semantics,generic_paint\n"
    ));
    assert!(text.contains(
        "authored-object index=0 id=\"canvas:node\" role=\"diagram_node\" name=\"Node\""
    ));
    assert!(text.contains(
        "authored-binding object_index=0 binding_index=1 object_id=\"canvas:node\" authored_binding_id=\"handle\""
    ));
    assert!(!text.contains("authored_binding_id=\"outline\""));
    assert!(!text.contains("node id=\"ak:1\""));
    assert!(!text.contains("paint order="));
}

#[test]
fn authored_focus_preserves_duplicate_object_matches() {
    let capture = duplicate_object_capture();
    let text = correlated_capture_authored_focus_to_agent_text(&capture, "duplicate", None);

    assert!(text.contains("object_match_count=2 binding_match_count=2"));
    assert_eq!(text.matches("authored-object index=").count(), 2);
    assert_eq!(text.matches("authored-binding object_index=").count(), 2);
    assert!(text.contains("authored-object index=0 id=\"duplicate\" role=\"first\""));
    assert!(text.contains("authored-object index=1 id=\"duplicate\" role=\"second\""));
}

#[test]
fn authored_focus_preserves_duplicate_binding_matches() {
    let mut capture = sample_capture();
    capture.authored_objects[0]
        .bindings
        .push(EguiAuthoredPaintBinding {
            authored_binding_id: Some("handle".into()),
            binding_evidence: "observed".into(),
            layer_order: EguiLayerOrder::Background,
            layer_id: 42,
            shape_index: 5,
            verified_at_end_pass: true,
            kind: Some(EguiPaintKind::Circle),
            bounds: Some(Rect {
                x: 50.0,
                y: 20.0,
                width: 10.0,
                height: 10.0,
            }),
            clip_rect: None,
        });

    let text =
        correlated_capture_authored_focus_to_agent_text(&capture, "canvas:node", Some("handle"));
    assert!(text.contains("object_match_count=1 binding_match_count=2"));
    assert_eq!(text.matches("authored_binding_id=\"handle\"").count(), 2);
}

#[test]
fn authored_focus_zero_match_is_explicit_not_an_error() {
    let capture = sample_capture();
    let text = correlated_capture_authored_focus_to_agent_text(&capture, "missing", None);

    assert!(text.contains("object_id=\"missing\" object_match_count=0 binding_match_count=0"));
    assert!(!text.contains("authored-object index="));
    assert!(!text.contains("authored-binding object_index="));
}

fn sample_capture() -> EguiCorrelatedCapture {
    let witness = from_yaml(
        r#"
viewwitness_version: "0"
capture:
  source: egui
  frame: 9
  viewport:
    width: 100.0
    height: 50.0
    scale_factor: 1.0
nodes:
  - id: "ak:1"
    role: button
    name: Apply
relations: []
"#,
    )
    .expect("parse focused projection witness");

    EguiCorrelatedCapture {
        request_id: 7,
        viewport_id: 2,
        pass_nr: 9,
        viewport_rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
        witness,
        paint: vec![EguiPaintObservation {
            order: 1,
            kind: EguiPaintKind::Rect,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            clip_rect: None,
        }],
        authored_objects: vec![EguiAuthoredPaintObject {
            id: "canvas:node".into(),
            role: "diagram_node".into(),
            name: Some("Node".into()),
            semantic_evidence: "intended".into(),
            bindings: vec![
                keyed_binding("outline", EguiPaintKind::Rect, 3, 20.0),
                keyed_binding("handle", EguiPaintKind::Circle, 4, 40.0),
            ],
        }],
    }
}

fn duplicate_object_capture() -> EguiCorrelatedCapture {
    let mut capture = sample_capture();
    capture.authored_objects = vec![
        EguiAuthoredPaintObject {
            id: "duplicate".into(),
            role: "first".into(),
            name: None,
            semantic_evidence: "intended".into(),
            bindings: vec![keyed_binding("part-a", EguiPaintKind::Rect, 0, 0.0)],
        },
        EguiAuthoredPaintObject {
            id: "duplicate".into(),
            role: "second".into(),
            name: None,
            semantic_evidence: "intended".into(),
            bindings: vec![keyed_binding("part-b", EguiPaintKind::Circle, 1, 5.0)],
        },
    ];
    capture
}

fn keyed_binding(
    id: &str,
    kind: EguiPaintKind,
    shape_index: usize,
    x: f32,
) -> EguiAuthoredPaintBinding {
    EguiAuthoredPaintBinding {
        authored_binding_id: Some(id.into()),
        binding_evidence: "observed".into(),
        layer_order: EguiLayerOrder::Background,
        layer_id: 42,
        shape_index,
        verified_at_end_pass: true,
        kind: Some(kind),
        bounds: Some(Rect {
            x,
            y: 20.0,
            width: 10.0,
            height: 10.0,
        }),
        clip_rect: None,
    }
}
