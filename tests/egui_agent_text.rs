#![cfg(feature = "egui")]

use viewwitness::{
    EguiCorrelatedCapture, EguiPaintKind, EguiPaintObservation, Rect,
    correlated_capture_to_agent_text, from_yaml,
};

#[test]
fn correlated_agent_text_keeps_semantics_and_paint_distinct_and_ordered() {
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
    };

    let text = correlated_capture_to_agent_text(&capture);
    let expected = concat!(
        "egui-correlated request=7 viewport_id=2 pass=9 viewport_rect=[0,0,100,50] paint_count=2 correlation=same_full_output\n",
        "view version=\"0\" source=\"egui\" viewport=[100,50,1] frame=9\n",
        "node id=\"ak:1\" role=\"button\" name=\"Apply\" bounds=[10,10,20,10]\n",
        "paint order=1 kind=\"circle\" bounds=[0,0,10,10] clip=[0,0,5,10] visible_fraction=0.5 visible_fraction_evidence=derived_bbox_clip visible_bounds=[0,0,5,10]\n",
        "paint order=2 kind=\"rect\" bounds=[0,0,10,10] clip=unbounded visible_fraction=1 visible_fraction_evidence=derived_bbox_clip visible_bounds=[0,0,10,10]\n",
    );

    assert_eq!(text, expected);
    assert!(
        !text.contains("node_paint") && !text.contains("paint_node"),
        "projection must not imply a semantic-node to paint-shape mapping"
    );
}
