use std::fs;

#[test]
fn canvas_showcase_keeps_four_explicit_authored_binding_keys() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase.rs");
    let source = fs::read_to_string(path).expect("read showcase source");

    for binding_id in ["outline", "handle", "ring", "center"] {
        let needle = format!("object.add_shape_with_id(\n                    &painter,\n                    \"{binding_id}\"");
        assert!(
            source.contains(&needle),
            "showcase Canvas must keep authored binding key {binding_id:?}"
        );
    }

    assert_eq!(
        source.matches("object.add_shape_with_id(").count(),
        4,
        "showcase Canvas should expose exactly four explicitly keyed paint bindings"
    );
    assert!(source.contains("showcase:painted-rectangle"));
    assert!(source.contains("showcase:painted-circle"));
}
