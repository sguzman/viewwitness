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
  source: synthetic
  frame: 42
  viewport: { width: 1280, height: 720, scale_factor: 1.0 }
  metadata: {}
nodes:
  - id: save
    role: button
    parent: toolbar
    name: Save
    bounds: { x: 12, y: 8, width: 72, height: 28 }
    visible: true
    enabled: true
    actions: [click]
relations:
  - kind: left_of
    from: save
    to: preview
    evidence: derived
    properties: {}
```

## Extension strategy

v0 deliberately leaves `role`, `actions`, relation `kind`, and the `properties` maps open. We should learn the necessary vocabulary from examples before replacing any of these with closed enums.

Capture-specific information belongs in `metadata` or node `properties` until it proves general enough for the core schema.

## Identity

Node IDs must be unique within a witness. Stable IDs across witnesses are strongly preferred because transition/diff tooling depends on them, but the capture backend may sometimes need an explicit identity strategy.

## Coordinate convention

Bounds use logical GUI coordinates with origin at the viewport's top-left. Width and height are non-negative. Future coordinate spaces must name themselves explicitly rather than silently changing this convention.

## Validation

The initial Rust validator checks:

- non-empty format version;
- unique node IDs;
- valid parent references;
- valid relation endpoints;
- non-negative viewport and rectangle dimensions;
- relation evidence is `observed` or `derived`.

Stronger semantic validation should arrive only alongside a firmer ontology.
