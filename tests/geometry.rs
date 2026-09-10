use viewwitness::{
    GeometryOptions, derive_geometry_relations, derive_geometry_relations_with_options, from_yaml,
};

const FIXTURE: &str = r#"
viewwitness_version: "0.1"
capture:
  source: synthetic
  viewport: { width: 1000, height: 700, scale_factor: 1.0 }
nodes:
  - { id: root, role: window, bounds: { x: 0, y: 0, width: 1000, height: 700 }, visible: true }
  - { id: sidebar, role: panel, parent: root, bounds: { x: 0, y: 40, width: 200, height: 660 }, visible: true }
  - { id: preview, role: canvas, parent: root, bounds: { x: 200, y: 40, width: 800, height: 660 }, visible: true }
  - { id: inspector, role: panel, parent: root, bounds: { x: 700, y: 60, width: 280, height: 620 }, visible: true }
  - { id: slider, role: slider, parent: inspector, bounds: { x: 720, y: 120, width: 240, height: 24 }, visible: true }
  - { id: hidden, role: panel, parent: root, bounds: { x: 700, y: 60, width: 280, height: 620 }, visible: false }
relations: []
"#;

#[test]
fn default_geometry_is_conservative_and_nonredundant() {
    let witness = from_yaml(FIXTURE).expect("fixture parses");
    let relations = derive_geometry_relations(&witness);

    assert!(has(&relations, "left_of", "sidebar", "preview"));
    assert!(has(&relations, "overlaps", "inspector", "preview"));
    assert!(!has(&relations, "right_of", "preview", "sidebar"));

    // Default derivation is sibling-only, so nested controls do not create a
    // redundant cloud of relations against their parent's siblings.
    assert!(!has(&relations, "overlaps", "preview", "slider"));

    // Explicitly hidden nodes are excluded by default.
    assert!(!relations.iter().any(|relation| {
        relation.from == "hidden" || relation.to == "hidden"
    }));
}

#[test]
fn overlap_carries_quantitative_evidence() {
    let witness = from_yaml(FIXTURE).expect("fixture parses");
    let relations = derive_geometry_relations(&witness);
    let overlap = relations
        .iter()
        .find(|relation| {
            relation.kind == "overlaps"
                && relation.from == "inspector"
                && relation.to == "preview"
        })
        .expect("inspector/preview overlap");

    assert_eq!(overlap.evidence, "derived");
    assert_eq!(overlap.properties["overlap_width"], 280.0);
    assert_eq!(overlap.properties["overlap_height"], 620.0);
    assert_eq!(overlap.properties["overlap_area"], 173_600.0);
}

#[test]
fn cross_parent_relations_can_be_requested_explicitly() {
    let witness = from_yaml(FIXTURE).expect("fixture parses");
    let options = GeometryOptions {
        siblings_only: false,
        ..GeometryOptions::default()
    };
    let relations = derive_geometry_relations_with_options(&witness, options);

    assert!(has(&relations, "overlaps", "preview", "slider"));
}

#[test]
fn symmetric_relation_order_is_stable_by_node_id() {
    let witness = from_yaml(
        r#"
viewwitness_version: "0.1"
capture:
  source: synthetic
  viewport: { width: 100, height: 100, scale_factor: 1.0 }
nodes:
  - { id: zeta, role: panel, bounds: { x: 0, y: 0, width: 80, height: 80 } }
  - { id: alpha, role: panel, bounds: { x: 10, y: 10, width: 80, height: 80 } }
relations: []
"#,
    )
    .expect("fixture parses");

    let relations = derive_geometry_relations(&witness);
    assert!(has(&relations, "overlaps", "alpha", "zeta"));
}

fn has(relations: &[viewwitness::Relation], kind: &str, from: &str, to: &str) -> bool {
    relations
        .iter()
        .any(|relation| relation.kind == kind && relation.from == from && relation.to == to)
}
