# egui integration

ViewWitness initially targets Rust + egui. The integration is intentionally split into two paths because a GUI witness is useful both inside tests and outside a running application.

## Semantic source

egui already produces AccessKit semantic output. ViewWitness consumes that output as observed evidence instead of inventing a second widget-semantic vocabulary at capture time.

The `egui` crate feature exposes:

- `EguiCaptureContext`;
- `witness_from_egui_output`;
- `witness_from_egui_tree_update`.

`witness_from_egui_tree_update` is specifically an **egui adapter**, not a generic AccessKit incremental-update consumer. egui's frame output provides a complete tree, which lets ViewWitness reconstruct parentage directly from that frame without retaining a second accessibility tree across calls.

AccessKit IDs are projected as `ak:<u64>` in v0. Captured egui nodes also carry explicit identity evidence:

```yaml
identity:
  provenance: accesskit_node_id
  stability: structure_sensitive
  author_id: optional-application-id
```

`structure_sensitive` is intentional. The executable egui identity probe shows that an automatically identified widget can retain its AccessKit identity across ordinary state changes while insertion of a preceding widget can change that identity. ViewWitness therefore treats the source ID as useful continuity evidence without claiming that it is a permanent conceptual identity.

When AccessKit exposes an application-authored `author_id`, ViewWitness preserves it as additional identity evidence. It does not silently substitute it for the observed AccessKit node ID.

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
- identity provenance/stability plus optional author ID;
- selected AccessKit properties such as busy/read-only/required/modal/expanded;
- semantic relations such as `labels`, `describes`, and `controls`.

Capture metadata records that AccessKit was the semantic source, records the egui identity policy, and preserves available tree/root/focus/toolkit identifiers.

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

The optional `observer` feature provides a read-only external `InspectionObserver` that speaks the versioned `egui_inspection` protocol directly. It does not depend on MCP.

The observer:

1. connects to the inspection endpoint;
2. validates the inspection-protocol handshake/version;
3. requests `GetTree`;
4. receives the complete AccessKit tree, inspection step, and pixels-per-point;
5. converts that evidence into the canonical `Witness` outside the GUI process.

The current observer derives viewport dimensions from observed root bounds. If the root does not provide usable bounds, it returns an error instead of inventing viewport geometry.

A loopback mock-peer integration test exercises the actual TCP handshake and MessagePack framing in CI, so the observer transport is tested without requiring a graphical desktop session.

Run the native showcase with inspection enabled in PowerShell:

```powershell
$env:EGUI_INSPECTION="1"
cargo run --example showcase --features showcase
```

Then, from another shell in the same repository:

```powershell
cargo run --example inspection_capture --features observer
```

The example defaults to `127.0.0.1:5719` and prints one live ViewWitness YAML document.

Keep inspection bound to loopback unless remote exposure is explicitly secured. The inspection protocol can expose GUI state and supports input/screenshot operations; ViewWitness's current observer uses only the read-only tree path, but the underlying endpoint should still be treated as a control surface.

The architecture is now:

```text
running egui/eframe app
        |
        | egui_inspection GetTree
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

The next real egui work should answer concrete questions through the living showcase and observer:

1. which ordinary egui widgets produce enough AccessKit information without extra instrumentation;
2. which visual facts require `WidgetInfo`, paint/layer information, clip rectangles, or application annotations;
3. how application-authored identity should improve diff matching without hiding heuristic reconciliation;
4. how much semantic information disappears for custom-painted/canvas content;
5. how a live external observer should merge semantic evidence with geometry/layer evidence without touching the render thread;
6. whether observed root bounds are a sufficiently reliable viewport source across native platforms and viewport configurations.
