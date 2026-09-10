#![cfg(feature = "egui")]

use egui::accesskit::{Action, Node as AccessNode, NodeId, Role, Tree, TreeId, TreeUpdate};
use serde_json::json;
use viewwitness::{
    EguiCaptureContext, Viewport, witness_from_egui_output, witness_from_egui_tree_update,
};

#[test]
fn accesskit_translation_preserves_structure_semantics_and_actions() {
    let root_id = NodeId(1);
    let label_id = NodeId(2);
    let field_id = NodeId(3);
    let button_id = NodeId(4);

    let mut root = AccessNode::new(Role::Window);
    root.set_label("Inspector");
    root.set_children(vec![label_id, field_id, button_id]);

    let mut label = AccessNode::new(Role::Label);
    label.set_value("Name");
    label.set_bounds(egui::accesskit::Rect {
        x0: 12.0,
        y0: 12.0,
        x1: 72.0,
        y1: 32.0,
    });

    let mut field = AccessNode::new(Role::TextInput);
    field.set_value("Cube");
    field.push_labelled_by(label_id);
    field.add_action(Action::Focus);
    field.add_action(Action::SetValue);
    field.set_bounds(egui::accesskit::Rect {
        x0: 80.0,
        y0: 8.0,
        x1: 240.0,
        y1: 36.0,
    });

    let mut button = AccessNode::new(Role::Button);
    button.set_label("Apply");
    button.set_disabled();
    button.add_action(Action::Click);
    button.set_bounds(egui::accesskit::Rect {
        x0: 160.0,
        y0: 48.0,
        x1: 240.0,
        y1: 78.0,
    });

    let update = TreeUpdate {
        nodes: vec![
            (root_id, root),
            (field_id, field),
            (button_id, button),
            (label_id, label),
        ],
        tree: Some(Tree::new(root_id)),
        tree_id: TreeId::ROOT,
        focus: field_id,
    };

    let witness = witness_from_egui_tree_update(
        &update,
        EguiCaptureContext::new(Viewport {
            width: 320.0,
            height: 200.0,
            scale_factor: 1.0,
        })
        .with_frame(7),
    );

    assert!(witness.validation_issues().is_empty());
    assert_eq!(witness.capture.source, "egui");
    assert_eq!(witness.capture.frame, Some(7));
    assert_eq!(witness.nodes.len(), 4);

    let label = witness
        .nodes
        .iter()
        .find(|node| node.id == "ak:2")
        .expect("label translated");
    assert_eq!(label.role, "label");
    assert_eq!(label.name.as_deref(), Some("Name"));
    assert_eq!(label.text.as_deref(), Some("Name"));
    assert_eq!(label.parent.as_deref(), Some("ak:1"));

    let field = witness
        .nodes
        .iter()
        .find(|node| node.id == "ak:3")
        .expect("field translated");
    assert_eq!(field.role, "textbox");
    assert_eq!(field.value, Some(json!("Cube")));
    assert_eq!(field.focused, Some(true));
    assert!(field.actions.iter().any(|action| action == "set_value"));

    let button = witness
        .nodes
        .iter()
        .find(|node| node.id == "ak:4")
        .expect("button translated");
    assert_eq!(button.role, "button");
    assert_eq!(button.name.as_deref(), Some("Apply"));
    assert_eq!(button.enabled, Some(false));
    assert!(button.actions.iter().any(|action| action == "click"));

    assert!(witness.relations.iter().any(|relation| {
        relation.kind == "labels"
            && relation.from == "ak:2"
            && relation.to == "ak:3"
            && relation.evidence == "observed"
    }));
}

#[test]
fn headless_egui_frame_becomes_a_valid_witness() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();

    let mut checked = true;
    let mut amount = 4.0_f32;
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 480.0),
            )),
            ..Default::default()
        },
        |ui| {
            ui.heading("ViewWitness showcase");
            let _ = ui.button("Save");
            ui.checkbox(&mut checked, "Autosave");
            ui.add(egui::Slider::new(&mut amount, 0.0..=10.0).text("Amount"));
        },
    );

    let witness = witness_from_egui_output(
        &output,
        EguiCaptureContext::new(Viewport {
            width: 640.0,
            height: 480.0,
            scale_factor: 999.0,
        })
        .with_frame(11),
    )
    .expect("AccessKit was enabled");

    assert!(witness.validation_issues().is_empty());
    assert_eq!(witness.capture.frame, Some(11));
    assert_eq!(
        witness.capture.viewport.scale_factor,
        output.pixels_per_point
    );

    let save = witness
        .nodes
        .iter()
        .find(|node| node.name.as_deref() == Some("Save"))
        .expect("egui button appears in witness");
    assert_eq!(save.role, "button");
    assert!(save.actions.iter().any(|action| action == "click"));
    assert!(save.bounds.is_some());

    let autosave = witness
        .nodes
        .iter()
        .find(|node| node.name.as_deref() == Some("Autosave"))
        .expect("egui checkbox appears in witness");
    assert_eq!(autosave.role, "checkbox");
    assert_eq!(autosave.properties.get("toggled"), Some(&json!("true")));

    output.drop_without_applying_deltas();
}
