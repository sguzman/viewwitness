# ViewWitness format v0

This is a draft executable specification, not a compatibility promise.

## Design constraints

A ViewWitness document should be:

1. readable by a human in a bug report;
2. cheap for an agent to parse and reason about;
3. lossless enough for structural/visual debugging without becoming a paint-command dump;
4. deterministic enough to diff;
5. explicit about provenance and epistemic status;
6. naturally representable by Rust + Serde.

YAML is the first showcase serialization because nested GUI state remains readable. The Rust model is canonical; YAML is a projection of it.

## Skeleton

```yaml
viewwitness_version: "0.1"
capture:
  source: egui
  frame: 42
  viewport: { width: 1280, height: 720, scale_factor: 1.0 }
  metadata:
    semantic_source: accesskit
nodes:
  - id: ak:1234
    role: button
    parent: ak:1000
    identity:
      provenance: accesskit_node_id
      stability: structure_sensitive
      author_id: save
    name: Save
    bounds: { x: 12, y: 8, width: 72, height: 28 }
    visible: true
    enabled: true
    actions: [click]
relations:
  - kind: left_of
    from: ak:1234
    to: ak:2000
    evidence: derived
    properties: {}
```

## Extension strategy

v0 deliberately leaves `role`, `actions`, relation `kind`, and the `properties` maps open. We should learn the necessary vocabulary from examples before replacing any of these with closed enums.

Capture-specific information belongs in `metadata` or node `properties` until it proves general enough for the core schema.

## Identity

`Node.id` is first an address inside one witness. It must be unique within that witness. Reusing the same string in two captures is useful evidence for continuity, but ViewWitness does **not** treat string equality alone as proof that the nodes are the same conceptual object forever.

A node may therefore carry an `identity` object:

- `provenance` — the mechanism that produced `Node.id`, such as `accesskit_node_id`;
- `stability` — an explicit statement about the cross-frame stability of that mechanism, such as `structure_sensitive`;
- `author_id` — optional application-authored identity evidence intended for automation/testing.

The distinction is deliberate. A backend-generated ID can be stable under ordinary state changes yet change after structural edits. An application-authored identifier may provide stronger semantic continuity, but v0 preserves it as additional evidence rather than silently replacing the captured node ID.

The egui adapter currently marks AccessKit-backed identity as `structure_sensitive`. Executable tests demonstrate that an ordinary state change can preserve a target widget's identity while insertion of a preceding widget can change an automatically generated egui/AccessKit identity.

Future diff matching may use stronger identity evidence, but heuristic reconciliation must remain distinguishable from direct source identity. v0 diffing still matches nodes by `Node.id`.

## Coordinate convention

Bounds use logical GUI coordinates with origin at the viewport's top-left. Width and height are non-negative. Future coordinate spaces must name themselves explicitly rather than silently changing this convention.

## Validation

The initial Rust validator checks:

- non-empty format version;
- unique node IDs;
- non-empty identity provenance/stability when identity evidence is present;
- non-empty `author_id` when supplied;
- valid parent references;
- valid relation endpoints;
- non-negative viewport and rectangle dimensions;
- relation evidence is `observed` or `derived`.

Stronger semantic validation should arrive only alongside a firmer ontology.
