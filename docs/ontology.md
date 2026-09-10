# ViewWitness mini-ontology — v0 exploration

ViewWitness needs enough ontology to make GUI evidence comparable without pretending to solve the ontology of every interface.

## Epistemic categories

The most important distinction is not a widget role. It is **how a fact is known**.

- **Observed** — supplied directly by the capture source. Examples: a node's bounds reported by egui, focus state, a label, an AccessKit role.
- **Derived** — computed deterministically from observed facts. Example: `panel overlaps viewport-content` derived from two rectangles.
- **Inferred** — a plausible interpretation requiring heuristics or model judgment. This category is intentionally *not yet representable as a normal relation*. If introduced later, it must remain distinguishable from observation and deterministic derivation.
- **Intended** — source/specification-level claims about what the UI should be. Not part of a basic witness.

A witness is testimony about observed interface state, not a restatement of source code intent.

A derived fact may depend on several observed facts. For example, an `occludes` relation could eventually be derived from rectangle intersection plus observed paint/layer order and clipping. It must not be derived from intersection alone merely because occlusion looks plausible.

## Core entities

### Witness

One capture of one GUI state. A witness owns capture provenance, viewport facts, nodes, and relations.

### Node

An addressable thing in the observed interface. A node can be semantic, visual, interactive, structural, or several at once.

This deliberately includes things that classic accessibility trees may not consider primary controls. A canvas object, selection outline, drag handle, or other agent-relevant visual affordance can deserve a node when it matters to describing or manipulating the observed interface.

Nodes use stable textual IDs when possible so related witnesses can be compared.

### Role

A semantic classification of a node. v0 keeps roles open-ended while examples pressure the vocabulary. Initial useful terms include:

`window`, `panel`, `toolbar`, `button`, `label`, `textbox`, `checkbox`, `radio`, `slider`, `list`, `listitem`, `tree`, `treeitem`, `table`, `row`, `cell`, `columnheader`, `scroll_area`, `viewport`, `canvas`, `image`, `menu`, `menuitem`, `dialog`, `tooltip`, `separator`, `status`, `paragraph`, `shape`, `selection_indicator`, `drag_handle`.

This vocabulary should borrow from accessibility standards where doing so avoids needless invention, but ViewWitness is not constrained to accessibility semantics.

### Bounds

An axis-aligned rectangle in logical GUI coordinates: `x`, `y`, `width`, `height`.

The first implementation deliberately avoids pretending this covers transforms, curved geometry, or every clipping model. Those should enter only when an example needs them.

Bounds establish geometry, not visibility by themselves. A rectangle can intersect another rectangle without proving that either node visually covers the other; z-order, clipping, and paint semantics also matter.

### Action

An interaction the node advertises as available, such as `click`, `type`, `toggle`, `expand`, `collapse`, `scroll`, `increment`, `decrement`, or `drag`.

An advertised action is an affordance claim about the captured state. It does not guarantee that an injected action will succeed under every external condition.

### Relation

A typed fact connecting two nodes. Initial relation candidates include:

- structural: `contains`;
- spatial: `left_of`, `right_of`, `above`, `below`, `overlaps`;
- visibility: `occludes`, `clips`;
- alignment: `aligned_left`, `aligned_right`, `aligned_top`, `aligned_bottom`;
- semantic: `labels`, `describes`, `controls`.

Relations carry an `evidence` field. Geometry-derived overlap must say `derived`; capture-native semantic linkage may say `observed`.

Not every logically equivalent inverse needs to be serialized. The current geometry engine emits one directional fact such as `left_of(a, b)` rather than also materializing `right_of(b, a)`. This keeps witnesses smaller without erasing meaning; consumers may invert known relation types when querying.

### Transient state

Menus, tooltips, popups, temporary drag affordances, and other short-lived nodes are first-class observations when they exist in the captured frame.

The corpus currently represents transience through open node properties. That is intentionally provisional. A dedicated lifetime/transience field should only be promoted into the core model after real egui capture shows that consumers need consistent semantics across backends and frames.

## Hierarchy versus geometry

Parentage is primarily structural/semantic. It must not be treated as a complete description of visual containment.

A popup may be structurally attached near the root while geometrically overlapping a control deeply nested in another subtree. Conversely, a child can have bounds extending outside its parent's rectangle because of scrolling, transforms, overflow, or a broken layout.

This is why geometry analysis is configurable. Sibling-only derivation is a useful low-noise default, while cross-parent analysis remains available for overlays and broader diagnostics.

## Things deliberately not settled

- whether parentage and `contains` should both exist long term;
- whether roles become a closed Rust enum;
- whether z-order belongs on nodes, relations, or capture-specific properties;
- how exact visual style should be represented;
- whether inferred facts deserve a first-class evidence category;
- what constitutes stable identity across immediate-mode frames;
- whether transient state deserves a core field rather than an open property;
- how clipping, transforms, and scroll-space coordinates should compose;
- whether spatial containment should use `contains` or a more explicit geometric relation name;
- how aggressively agent-oriented serializers should prune derivable relations.

The example corpus exists to answer these questions by pressure rather than taste alone.
