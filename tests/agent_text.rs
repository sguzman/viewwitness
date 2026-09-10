use viewwitness::{diff_to_agent_text, diff_witnesses, from_yaml, to_agent_text};

#[test]
fn witness_agent_text_is_compact_deterministic_and_complete() {
    let witness = from_yaml(
        r#"
viewwitness_version: "0.1"
capture:
  source: synthetic
  frame: 7
  viewport: { width: 200, height: 100, scale_factor: 1.0 }
  metadata:
    z: 2
    a: x
nodes:
  - id: b-save
    role: button
    parent: a-root
    identity:
      provenance: test
      stability: authored
      author_id: save
    name: Save now
    bounds: { x: 10, y: 10, width: 80, height: 24 }
    visible: true
    enabled: false
    focused: false
    actions: [click]
    properties: { busy: true }
  - id: a-root
    role: window
    name: Main
    bounds: { x: 0, y: 0, width: 200, height: 100 }
    visible: true
relations:
  - kind: controls
    from: a-root
    to: b-save
    evidence: observed
    properties: {}
"#,
    )
    .expect("fixture parses");

    assert_eq!(
        to_agent_text(&witness),
        concat!(
            "view version=\"0.1\" source=\"synthetic\" viewport=[200,100,1] frame=7\n",
            "meta {\"a\":\"x\",\"z\":2}\n",
            "node id=\"a-root\" role=\"window\" name=\"Main\" bounds=[0,0,200,100] visible=true\n",
            "node id=\"b-save\" role=\"button\" parent=\"a-root\" identity={\"provenance\":\"test\",\"stability\":\"authored\",\"author_id\":\"save\"} name=\"Save now\" bounds=[10,10,80,24] visible=true enabled=false focused=false actions=[\"click\"] props={\"busy\":true}\n",
            "relation kind=\"controls\" from=\"a-root\" to=\"b-save\" evidence=\"observed\"\n",
        )
    );
}

#[test]
fn diff_agent_text_reports_only_material_transition_lines() {
    let before = from_yaml(
        r#"
viewwitness_version: "0.1"
capture:
  source: synthetic
  frame: 1
  viewport: { width: 200, height: 100, scale_factor: 1.0 }
nodes:
  - id: root
    role: window
  - id: export
    role: button
    parent: root
    name: Export
    enabled: true
    actions: [click]
relations: []
"#,
    )
    .expect("before parses");
    let after = from_yaml(
        r#"
viewwitness_version: "0.1"
capture:
  source: synthetic
  frame: 2
  viewport: { width: 200, height: 100, scale_factor: 1.0 }
nodes:
  - id: root
    role: window
  - id: export
    role: button
    parent: root
    name: Export
    enabled: false
    actions: []
  - id: status
    role: status
    parent: root
    name: Exporting
relations:
  - kind: describes
    from: status
    to: export
    evidence: observed
    properties: {}
"#,
    )
    .expect("after parses");

    let diff = diff_witnesses(&before, &after);
    assert_eq!(
        diff_to_agent_text(&diff),
        concat!(
            "diff before_frame=1 after_frame=2\n",
            "+node id=\"status\" role=\"status\" parent=\"root\" name=\"Exporting\"\n",
            "change node=\"export\" field=\"actions\" before=[\"click\"] after=[]\n",
            "change node=\"export\" field=\"enabled\" before=true after=false\n",
            "+relation kind=\"describes\" from=\"status\" to=\"export\" evidence=\"observed\"\n",
        )
    );
}
