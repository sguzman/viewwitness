# ViewWitness mini-ontology — v0 exploration

ViewWitness needs enough ontology to make GUI evidence comparable without pretending to solve the ontology of every interface.

## Epistemic categories

The most important distinction is not a widget role. It is **how a fact is known**.

- **Observed** — supplied directly by the capture source. Examples: a node's bounds reported by egui, focus state, a label, an AccessKit role.
- **Derived** — computed deterministically from observed facts. Example: `panel overlaps viewport-content` derived from two rectangles.
- **Inferred** — a plausible interpretation requiring heuristics or model judgment. This category is intentionally *not yet representable as a normal relation*. If introduced later, it must remain distinguishable from observation and deterministic derivation.
- **Intended** — source/specification-level claims about what the UI should be. Not part of a basic witness.

A witness is testimony about observed interface state, not a restatement of source code intent.

## Core entities

### Witness

One capture of one GUI state. A witness owns capture provenance, viewport facts, nodes, and relations.

### Node

An addressable thing in the observed interface. A node can be semantic, visual, interactive, structural, or several at once.

Nodes use stable textual IDs when possible so related witnesses can be compared.

### Role

A semantic classification of a node. v0 keeps roles open-ended while examples pressure the vocabulary. Initial useful terms include:

`window`, `panel`, `toolbar`, `button`, `label`, `textbox`, `checkbox`, `radio`, `slider`, `list`, `listitem`, `tree`, `treeitem`, `scroll_area`, `viewport`, `canvas`, `image`, `menu`, `menuitem`, `dialog`, `separator`, `status`.

This vocabulary should borrow from accessibility standards where doing so avoids needless invention, but ViewWitness is not constrained to accessibility semantics.

### Bounds

An axis-aligned rectangle in logical GUI coordinates: `x`, `y`, `width`, `height`.

The first implementation deliberately avoids pretending this covers transforms, curved geometry, or every clipping model. Those should enter only when an example needs them.

### Action

An interaction the node advertises as available, such as `click`, `type`, `toggle`, `expand`, `collapse`, `scroll`, `increment`, or `decrement`.

### Relation

A typed fact connecting two nodes. Initial relation candidates include:

- structural: `contains`;
- spatial: `left_of`, `right_of`, `above`, `below`, `overlaps`;
- visibility: `occludes`, `clips`;
- alignment: `aligned_left`, `aligned_right`, `aligned_top`, `aligned_bottom`;
- semantic: `labels`, `describes`, `controls`.

Relations carry an `evidence` field. Geometry-derived overlap must say `derived`; capture-native semantic linkage may say `observed`.

## Things deliberately not settled

- whether parentage and `contains` should both exist long term;
- whether roles become a closed Rust enum;
- whether z-order belongs on nodes, relations, or capture-specific properties;
- how exact visual style should be represented;
- whether inferred facts deserve a first-class evidence category;
- what constitutes stable identity across immediate-mode frames;
- how to represent transient nodes that exist for only one frame;
- how clipping, transforms, and scroll-space coordinates should compose.

The example corpus exists to answer these questions by pressure rather than taste alone.
