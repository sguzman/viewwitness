use std::fs;

use viewwitness::{
    GeometryOptions, derive_geometry_relations, derive_geometry_relations_with_options, from_yaml,
};

#[test]
fn pathological_editor_rederives_its_overlap() {
    let witness = load("examples/snapshots/10-pathological-overlap.yaml");
    let derived = derive_geometry_relations(&witness);
    let overlap = derived
        .iter()
        .find(|relation| {
            relation.kind == "overlaps"
                && relation.from == "inspector"
                && relation.to == "preview"
        })
        .expect("inspector should overlap preview");

    assert_eq!(overlap.properties["overlap_width"], 570.0);
    assert_eq!(overlap.properties["overlap_height"], 646.0);
    assert_eq!(overlap.properties["overlap_area"], 368_220.0);
}

#[test]
fn popup_overlap_requires_cross_parent_analysis() {
    let witness = load("examples/snapshots/04-overlay-occlusion.yaml");

    let default_relations = derive_geometry_relations(&witness);
    assert!(!default_relations.iter().any(|relation| {
        relation.kind == "overlaps"
            && relation.from == "popup"
            && relation.to == "row-under-popup"
    }));

    let relations = derive_geometry_relations_with_options(
        &witness,
        GeometryOptions {
            siblings_only: false,
            ..GeometryOptions::default()
        },
    );
    let overlap = relations
        .iter()
        .find(|relation| {
            relation.kind == "overlaps"
                && relation.from == "popup"
                && relation.to == "row-under-popup"
        })
        .expect("cross-parent analysis should recover popup overlap");

    assert_eq!(overlap.properties["overlap_area"], 5_400.0);
}

fn load(path: &str) -> viewwitness::Witness {
    let source = fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    from_yaml(&source).unwrap_or_else(|error| panic!("failed to parse {path}: {error}"))
}
