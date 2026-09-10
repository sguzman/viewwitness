# egui integration

ViewWitness initially targets Rust + egui. The integration is intentionally split into two paths because a GUI witness is useful both inside tests and outside a running application.

## Semantic source

egui already produces AccessKit semantic output. ViewWitness consumes that output as observed evidence instead of inventing a second widget-semantic vocabulary at capture time.

The `egui` crate feature exposes:

- `EguiCaptureContext`;
- `witness_from_egui_output`;
- `witness_from_egui_tree_update`.

`witness_from_egui_tree_update` is specifically an **egui adapter**, not a generic AccessKit incremental-update consumer. egui's frame output provides a complete tree, which lets ViewWitness reconstruct parentage directly from that frame without retaining a second accessibility tree across calls.

AccessKit IDs are projected as `ak:<u64>` in v0. They are stable only to the extent the producer keeps the underlying AccessKit identity stable. ViewWitness must not claim stronger cross-frame identity than its source provides.

## What is mapped today

The first adapter slice preserves:

- semantic role;
- label/name and textual or numeric value;
- parent/child structure;
- logical bounds;
- visible/hidden state;
- disabled/enabled state when meaningful;
- focus;
- selection;
- toggled state;
- advertised actions;
- selected AccessKit properties such as busy/read-only/required/modal/expanded;
- semantic relations such as `labels`, `describes`, and `controls`.

Capture metadata records that AccessKit was the semantic source and preserves available tree/root/focus/toolkit identifiers.

The adapter intentionally does **not** pretend AccessKit alone exhausts visual state. Paint order, exact styling, clip stacks, non-accessibility canvas primitives, and other rendered facts will require additional egui evidence or explicit application instrumentation.

## Coordinate caution

AccessKit nodes can carry transforms. v0 records `accesskit_transform_present: true` when one exists, but does not yet compose transforms into canonical ViewWitness bounds. Consumers should therefore treat transformed-node geometry as provisional until transform handling is implemented and tested.

Likewise, `clips_children` is preserved as observed semantic evidence, but ViewWitness does not yet derive precise clipping or visible fractions from it. Robust clipping requires explicit coordinate-space and clip-rectangle semantics.

## Two capture modes

### Headless / in-process frame conversion

Tests and custom integrations can enable AccessKit on an `egui::Context`, produce a `FullOutput`, and immediately translate that frame into a `Witness`.

This path is ideal for:

- deterministic unit/integration tests;
- fixture generation;
- small custom applications that explicitly request a witness;
- development of the canonical mapping itself.

It should remain lightweight. Capturing semantic output is acceptable on the UI path; expensive serialization, diffing, searching, model inference, persistence, or network serving is not.

### Live eframe observation

For normal running applications, ViewWitness should prefer an **external observer** over performing heavy witness work inside the GUI update/render path.

Current eframe inspection support can expose a running application's AccessKit tree and input/screenshot capabilities over a local inspection connection. The intended ViewWitness architecture is therefore:

```text
running egui/eframe app
        |
        | inspection / semantic state
        v
external ViewWitness observer
        |
        +--> canonical Witness
        +--> derived relations
        +--> YAML / compact text
        +--> diffs
        +--> future agent bridge
```

This preserves a core project rule: **the GUI thread reports GUI state; it does not become the worker responsible for analyzing that state.**

## Agent boundary

MCP is not the ViewWitness data model. egui inspection is not the ViewWitness data model. AccessKit is not the ViewWitness data model.

They are integration surfaces and evidence sources around the canonical `Witness` representation.

Keeping those layers separate lets the current egui-first implementation exploit mature tooling without making the ontology or serialized format hostage to any one transport.

## Next pressure points

The next real egui work should answer concrete questions through the living showcase:

1. which ordinary egui widgets produce enough AccessKit information without extra instrumentation;
2. which visual facts require `WidgetInfo`, paint/layer information, clip rectangles, or application annotations;
3. how stable egui/AccessKit IDs are across representative state transitions;
4. how much semantic information disappears for custom-painted/canvas content;
5. how a live external observer should merge semantic evidence with geometry/layer evidence without touching the render thread.
