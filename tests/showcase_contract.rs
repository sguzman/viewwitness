use std::fs;

#[test]
fn canvas_showcase_keeps_four_explicit_authored_binding_keys() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase.rs");
    let source = fs::read_to_string(path).expect("read showcase source");

    for binding_id in ["outline", "handle", "ring"] {
        let needle = format!(
            "object.add_shape_with_id(\n                    &painter,\n                    \"{binding_id}\""
        );
        assert!(
            source.contains(&needle),
            "showcase Canvas must keep authored binding key {binding_id:?} on the ordinary painter"
        );
    }

    assert!(source.contains(
        "object.add_shape_with_id(\n                    &center_painter,\n                    \"center\""
    ));
    assert_eq!(
        source.matches("object.add_shape_with_id(").count(),
        4,
        "showcase Canvas should expose exactly four explicitly keyed paint bindings"
    );
    assert!(source.contains("showcase:painted-rectangle"));
    assert!(source.contains("showcase:painted-circle"));
}

#[test]
fn canvas_showcase_keeps_a_live_misplaced_handle_pressure_state() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase.rs");
    let source = fs::read_to_string(path).expect("read showcase source");

    assert!(source.contains("Misplaced canvas handle"));
    assert!(source.contains("misplaced_canvas_handle"));
    assert!(source.contains("first.right_center() + egui::vec2(60.0, 0.0)"));
    assert!(source.contains(
        "Pressure defect active: the keyed rectangle handle is intentionally displaced 60 px to the right."
    ));
}

#[test]
fn canvas_showcase_keeps_a_live_clipped_center_pressure_state() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase.rs");
    let source = fs::read_to_string(path).expect("read showcase source");

    assert!(source.contains("Clip canvas center"));
    assert!(source.contains("clipped_canvas_center"));
    assert!(
        source.contains("painter.with_clip_rect(egui::Rect::from_min_max(rect.min, rect.min))")
    );
    assert!(source.contains(
        "Pressure defect active: the keyed circle center is intentionally clipped to zero visible area."
    ));
}

#[test]
fn pressure_states_are_deterministically_launchable() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase.rs");
    let source = fs::read_to_string(path).expect("read showcase source");

    assert!(source.contains("VIEWWITNESS_SHOWCASE_SCENARIO"));
    assert!(source.contains("Some(\"misplaced-handle\")"));
    assert!(source.contains("(ShowcasePage::Canvas, true, false)"));
    assert!(source.contains("ViewWitness showcase startup scenario: misplaced-handle"));
    assert!(source.contains("Some(\"clipped-center\")"));
    assert!(source.contains("(ShowcasePage::Canvas, false, true)"));
    assert!(source.contains("ViewWitness showcase startup scenario: clipped-center"));
}
