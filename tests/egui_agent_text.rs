#![cfg(feature = "egui")]

use viewwitness::{
    EguiAuthoredPaintBinding, EguiAuthoredPaintObject, EguiCorrelatedCapture, EguiLayerOrder,
    EguiPaintKind, EguiPaintObservation, Rect, correlated_capture_to_agent_text, from_yaml,
};

#[test]
fn correlated_agent_text_keeps_semantics_authored_objects_bindings_and_paint_distinct() {
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
    bounds:
      x: 10.0
      y: 10.0
      width: 20.0
      height: 10.0
relations: []
"#,
    )
    .expect("parse semantic witness");

    let capture = EguiCorrelatedCapture {
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
        paint: vec![
            EguiPaintObservation {
                order: 2,
                kind: EguiPaintKind::Rect,
                bounds: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                clip_rect: None,
            },
            EguiPaintObservation {
                order: 1,
                kind: EguiPaintKind::Circle,
                bounds: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                clip_rect: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 5.0,
                    height: 10.0,
                }),
            },
        ],
        authored_objects: vec![EguiAuthoredPaintObject {
            id: "canvas:node".into(),
            role: "diagram_node".into(),
            name: Some("Node".into()),
            semantic_evidence: "intended".into(),
            bindings: vec![
                EguiAuthoredPaintBinding {
                    binding_evidence: "observed".into(),
                    layer_order: EguiLayerOrder::Background,
                    layer_id: 42,
                    shape_index: 3,
                    verified_at_end_pass: true,
                    kind: Some(EguiPaintKind::Rect),
                    bounds: Some(Rect {
                        x: 20.0,
                        y: 20.0,
                        width: 30.0,
                        height: 20.0,
                    }),
                    clip_rect: None,
                },
                EguiAuthoredPaintBinding {
                    binding_evidence: "observed".into(),
                    layer_order: EguiLayerOrder::Background,
                    layer_id: 42,
                    shape_index: 4,
                    verified_at_end_pass: true,
                    kind: Some(EguiPaintKind::Circle),
                    bounds: Some(Rect {
                        x: 30.0,
                        y: 20.0,
                        width: 20.0,
                        height: 20.0,
                    }),
                    clip_rect: Some(Rect {
                        x: 30.0,
                        y: 20.0,
                        width: 10.0,
                        height: 20.0,
                    }),
                },
            ],
        }],
    };

    let text = correlated_capture_to_agent_text(&capture);
    let expected = concat!(
        "egui-correlated request=7 viewport_id=2 pass=9 viewport_rect=[0,0,100,50] paint_count=2 authored_count=1 authored_binding_count=2 correlation=same_full_output\n",
        "view version=\"0\" source=\"egui\" viewport=[100,50,1] frame=9\n",
        "node id=\"ak:1\" role=\"button\" name=\"Apply\" bounds=[10,10,20,10]\n",
        "authored-object index=0 id=\"canvas:node\" role=\"diagram_node\" name=\"Node\" semantic_evidence=\"intended\" binding_count=2\n",
        "authored-binding object_index=0 binding_index=0 object_id=\"canvas:node\" binding_evidence=\"observed\" layer_order=\"background\" layer_id=42 shape_index=3 verified=true kind=\"rect\" bounds=[20,20,30,20] clip=unbounded visible_fraction=1 visible_fraction_evidence=derived_bbox_clip visible_bounds=[20,20,30,20]\n",
        "authored-binding object_index=0 binding_index=1 object_id=\"canvas:node\" binding_evidence=\"observed\" layer_order=\"background\" layer_id=42 shape_index=4 verified=true kind=\"circle\" bounds=[30,20,20,20] clip=[30,20,10,20] visible_fraction=0.5 visible_fraction_evidence=derived_bbox_clip visible_bounds=[30,20,10,20]\n",
        "paint order=1 kind=\"circle\" bounds=[0,0,10,10] clip=[0,0,5,10] visible_fraction=0.5 visible_fraction_evidence=derived_bbox_clip visible_bounds=[0,0,5,10]\n",
        "paint order=2 kind=\"rect\" bounds=[0,0,10,10] clip=unbounded visible_fraction=1 visible_fraction_evidence=derived_bbox_clip visible_bounds=[0,0,10,10]\n",
    );

    assert_eq!(text, expected);
    assert!(
        !text.contains("node_paint") && !text.contains("paint_node"),
        "projection must not imply an AccessKit-node to paint-shape mapping"
    );
}

#[test]
fn duplicate_authored_ids_remain_separate_explicit_objects() {
    let witness = from_yaml(
        r#"
viewwitness_version: "0"
capture:
  source: egui
  viewport:
    width: 10.0
    height: 10.0
    scale_factor: 1.0
nodes: []
relations: []
"#,
    )
    .expect("parse semantic witness");

    let binding = |shape_index| EguiAuthoredPaintBinding {
        binding_evidence: "observed".into(),
        layer_order: EguiLayerOrder::Background,
        layer_id: 1,
        shape_index,
        verified_at_end_pass: true,
        kind: Some(EguiPaintKind::Rect),
        bounds: Some(Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }),
        clip_rect: None,
    };

    let capture = EguiCorrelatedCapture {
        request_id: 1,
        viewport_id: 1,
        pass_nr: 1,
        viewport_rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
        witness,
        paint: vec![],
        authored_objects: vec![
            EguiAuthoredPaintObject {
                id: "duplicate".into(),
                role: "first".into(),
                name: None,
                semantic_evidence: "intended".into(),
                bindings: vec![binding(0)],
            },
            EguiAuthoredPaintObject {
                id: "duplicate".into(),
                role: "second".into(),
                name: None,
                semantic_evidence: "intended".into(),
                bindings: vec![binding(1)],
            },
        ],
    };

    let text = correlated_capture_to_agent_text(&capture);
    assert!(text.contains("authored_count=2 authored_binding_count=2"));
    assert_eq!(text.matches("authored-object index=").count(), 2);
    assert!(text.contains("authored-object index=0 id=\"duplicate\" role=\"first\""));
    assert!(text.contains("authored-object index=1 id=\"duplicate\" role=\"second\""));
    assert_eq!(text.matches("object_id=\"duplicate\"").count(), 2);
}
