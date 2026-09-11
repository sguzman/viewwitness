# ViewWitness

**A Rust-first witness format for graphical user interfaces.**

ViewWitness turns observed GUI state into structured, serializable, diffable evidence that humans, tests, tools, and software agents can reason about.

The project begins deliberately narrow: **Rust + egui**. Broader GUI backends and language bindings are allowed by the architecture, but they are not current work.

## Why

Screenshots are rich but expensive to interpret. Accessibility trees are semantically useful but do not fully describe rendered reality. Source code describes intent, not necessarily what appeared on screen.

ViewWitness aims to preserve the useful intersection:

- semantic identity and hierarchy;
- actual geometry and visibility;
- interaction state and affordances;
- salient rendered evidence;
- spatial and semantic relationships;
- capture provenance;
- explicit identity provenance and stability;
- explicit distinction among observed, derived, inferred, and intended facts;
- deterministic textual serialization;
- state-to-state diffs suitable for debugging and agent verification.

The long-term test is simple: an agent should be able to inspect a broken egui application, explain what is visibly wrong, change the code, capture another witness, and demonstrate from the resulting state that the defect changed or disappeared.

## Evidence layers

ViewWitness deliberately refuses to collapse different kinds of GUI testimony into one object merely because they describe the same application.

Today the egui work distinguishes:

- **semantic evidence** — AccessKit roles, hierarchy, labels, values, actions, states, relations, and logical bounds;
- **paint-submission evidence** — renderer-facing egui shapes, flattened paint order, visual bounds, and clip rectangles;
- **derived clip evidence** — bounding-box survival through an observed clip rectangle;
- **authored custom-paint evidence** — application-declared logical-object semantics with one or more explicitly grouped, end-of-pass-verified egui paint bindings;
- **raster evidence** — screenshots returned by the inspected application.

A semantic node is not automatically a paint primitive. Rectangle overlap is not automatically occlusion. Application-authored meaning is not automatically observed meaning. Repeating an authored ID is not automatically object grouping. A screenshot is not automatically frame-correlated with a separately captured semantic tree. ViewWitness records those distinctions instead of guessing across them.

## Repository shape

- `src/` — canonical Rust model, serialization, geometry derivation, egui evidence adapters/reporters, exact capture, and optional live observers.
- `docs/ontology.md` — the small vocabulary ViewWitness commits to.
- `docs/format-v0.md` — draft wire-format contract.
- `docs/geometry.md` — deterministic geometry relation semantics.
- `docs/egui.md` — egui/AccessKit/rendered-evidence architecture and UI-thread boundary.
- `docs/rendered-evidence.md` — executable findings about paint, clipping, visual epistemics, and authored custom paint.
- `examples/snapshots/` — a growing corpus of representative GUI witnesses.
- `examples/transitions/` — before/after witnesses for state-transition and diff work.
- `examples/egui_capture.rs` — real headless egui frame → ViewWitness → YAML.
- `examples/inspection_capture.rs` — running inspected egui app → external ViewWitness observer → YAML.
- `examples/showcase.rs` — native eframe pressure surface for capture and diagnosis.
- `tests/` — executable checks over the corpus, actual egui output, diffs, identity behavior, backpressure, custom paint, and live protocol framing.

## Current egui slice

The optional `egui` feature translates egui-produced AccessKit output into the canonical ViewWitness model. It preserves semantic hierarchy, roles, names/values, bounds, visibility, focus, selection/toggle state, actions, selected semantic relations, and explicit identity evidence.

It also exposes provisional renderer evidence through `EguiPaintObservation`. Paint classification uses a small `Copy` enum rather than allocating a string per shape. Bounding-box clipping is derived explicitly; stronger claims such as true visual occlusion are not invented from rectangle overlap.

`EguiPaintReporter` provides continuous low-cost paint observation through a bounded nonblocking queue. The render/output hook performs cheap copying and `try_send` only; queue saturation drops evidence rather than stalling egui. A worker-side loopback TCP transport can stream those paint frames to an external read-only observer on `127.0.0.1:5720`.

For heavier diagnosis, `EguiFrameProbe` provides **on-demand exact same-pass capture**. Capture eligibility is fixed at pass start, then the requested pass supplies:

- the AccessKit update;
- egui's observed viewport rectangle;
- viewport identity;
- cumulative pass number;
- pixels-per-point scale;
- renderer-facing paint observations;
- explicit authored custom-paint objects and bindings, when the application supplies them.

The raw evidence is handed through a bounded channel and converted off the GUI thread into `EguiCorrelatedCapture`: a canonical semantic `Witness` plus provisional egui-specific evidence known to originate from the same requested pass.

The correlated path does **not** derive viewport size from AccessKit root bounds. Executable testing demonstrated that a valid headless egui semantic tree can lack usable root bounds even while egui itself has trustworthy viewport geometry. The exact probe therefore records `InputState::viewport_rect()` directly and rejects invalid geometry instead of guessing.

### Explicit custom-paint identity

Custom canvas graphics may have no useful AccessKit identity. ViewWitness supports a narrow, explicit solution without post-hoc inference.

The authored layer now has two different entities:

```text
EguiAuthoredPaintObject
    id / role / name / semantic_evidence
    bindings[]

EguiAuthoredPaintBinding
    binding_evidence
    LayerId + ShapeIdx
    verified_at_end_pass
    final kind / bounds / clip
```

Object ID, role, and optional name are application-declared **intended semantics**. Each binding is separate **observed execution evidence** tied to a concrete egui paint slot.

`EguiPaintAnnotator::paint_object(...)` is the explicit grouping operation. Its scoped callback can add several shapes and bind all of their actual egui handles to one logical object. The older one-shape `add_shape(...)` form remains a convenience and produces an object with one binding.

ViewWitness deliberately does **not** merge independent annotations merely because they repeat the same object ID. Executable tests create two independent objects with identical authored IDs and prove they remain separate records. Multi-shape membership exists only because the application explicitly grouped those submissions.

At `Plugin::on_end_pass`, after ordinary UI painting but before egui drains graphic layers into `FullOutput`, every binding is resolved against its actual layer-local slot. Tests prove:

- two objects with identical overlapping geometry remain distinct by real paint handle;
- an annotated rectangle later replaced through `Painter::set` is observed as the final **circle** occupying that slot;
- one explicit logical object can own multiple concrete paint handles;
- constituent bindings may have identical visual bounds while retaining different kinds and clipping;
- annotations from ordinary unrequested frames do not leak into later exact captures.

Clip survival belongs to the **binding**, not the logical object. `EguiAuthoredPaintBinding::visible_bounds()` / `visible_fraction()` derive bounding-box survival from each final verified binding. In the multi-shape pressure test, one logical object has two bindings with the same 40×40 final bounds: one survives fully while the other is half-clipped and reports `0.5`. ViewWitness does not invent one aggregate object-level visibility number from those conflicting facts.

This does **not** solve the generic AccessKit-node ↔ paint-shape problem. Ordinary widgets and unannotated paint remain separate evidence unless a source explicitly supplies their identity relationship.

## Exact external capture protocol

`run_egui_capture_server` and `EguiCaptureObserver` expose the exact product through the separate read-only request/response endpoint on `127.0.0.1:5721`.

The exact-capture handshake is currently:

```text
VIEWWITNESS-EGUI-CAPTURE 2
```

Protocol **v2** corresponds to the authored-object envelope described above: one logical authored object contains an explicit `bindings[]` list. This was a real wire-shape change from v1, so ViewWitness bumped the protocol instead of silently assigning new meaning to the same handshake. Executable tests require a v2 observer to reject an incompatible v1 peer.

`EguiCorrelatedCapture` has two external projections:

- a complete serializable envelope containing the canonical witness, correlated generic paint, and authored objects with their bindings;
- deterministic agent text that emits semantic witness lines, authored-object lines, authored-binding lines, and generic paint-submission lines separately.

## CLI

The ordinary semantic path remains available through upstream inspection:

```text
cargo run --features observer --bin viewwitness -- capture --settle=8
```

An application hosting ViewWitness's exact capture service can instead be queried with:

```text
cargo run --features egui --bin viewwitness -- capture-exact
```

Agent text is the default. Preserve the complete structured envelope as YAML with:

```text
cargo run --features egui --bin viewwitness -- capture-exact --yaml
```

Add deterministic semantic geometry relations without mutating the observed paint evidence with:

```text
cargo run --features egui --bin viewwitness -- capture-exact --derive
```

The exact command defaults to `127.0.0.1:5721`; a different address may be supplied positionally.

Run the living native pressure surface with:

```text
cargo run --example showcase --features showcase
```

Its ViewWitness services are installed during eframe application creation, while blocking network loops run on worker threads. The custom Canvas page now contains **two authored logical objects with four verified paint bindings**: the rectangle has an outline plus a handle, and the circle has a ring plus a center marker. Canvas background and text labels remain anonymous generic paint by design.

The optional `observer` feature speaks the versioned `egui_inspection` protocol directly from an external process. With an inspected eframe application running, this prints one live semantic witness:

```text
cargo run --example inspection_capture --features observer
```

The observer paths are deliberately external: GUI code reports state, while ViewWitness performs conversion, serialization, diffing, derivation, networking, and agent-facing work outside the render/update thread.

AccessKit/egui identity is treated as evidence rather than absolute truth. Automatically generated identity has been empirically shown to survive ordinary state changes while remaining sensitive to structural insertion, so captured nodes state that stability explicitly. Application-authored IDs are preserved as additional evidence when available.

## Two live tempos

The current architecture intentionally has two different observation tempos:

1. **continuous monitoring** — cheap generic paint metadata flows through a bounded queue; dropped observations are preferable to render-thread backpressure;
2. **exact diagnosis** — a requested pass copies semantic + viewport + paint evidence and optionally records explicitly authored custom-paint object/binding identity, then performs canonical conversion and network serialization off-thread.

These should not be collapsed into one mechanism. Continuous evidence needs to be cheap and disposable. Rich authored identity bookkeeping is activated only for an explicitly requested exact capture.

Likewise, the external `egui_inspection` semantic stream and ViewWitness paint stream use different clocks. `egui_inspection` owns its own plugin-global `step`; paint reporting uses egui's viewport-aware cumulative pass number. ViewWitness never joins them by equality or by an assumed fixed offset.

## Status

ViewWitness has moved beyond format-only exploration. The current project has:

- an executable synthetic snapshot corpus and transition corpus;
- deterministic geometry derivation;
- first-class `WitnessDiff` with YAML and compact agent-text projections;
- real egui/AccessKit capture translation behind an optional feature;
- explicit node identity provenance/stability;
- headless tests against actual egui output, including cross-frame identity behavior;
- provisional renderer-facing paint evidence with clip-survival derivation;
- a bounded nonblocking paint reporter with executable render-thread backpressure tests;
- a read-only external paint side channel with versioned handshake and latest-frame replay;
- an on-demand exact same-pass semantic + viewport + paint frame probe;
- an off-thread `EguiCorrelatedCapture` product with explicit frame-clock and correlation provenance;
- exact-capture protocol v2 and external `EguiCaptureObserver`;
- deterministic correlated agent text preserving semantic, authored-object, authored-binding, and generic-paint distinctions;
- `viewwitness capture-exact` with agent-text and full-envelope YAML output;
- explicit authored logical objects with one or many verified egui layer-local paint bindings;
- executable tests proving handle identity survives identical geometry and observes `Painter::set` replacement;
- explicit tests proving duplicate authored IDs do not create implicit grouping;
- per-binding clip visibility backed by final end-of-pass bounds + clip evidence;
- a living native eframe showcase with worker-hosted `:5720` and `:5721` services and a two-object/four-binding Canvas pressure case;
- a read-only external `egui_inspection` semantic/raster observer;
- loopback integration tests for live protocol framing and exact CLI capture;
- CI over both the default and all-features builds.

The v0 canonical `Witness` schema is still intentionally provisional. Generic paint and authored-object/binding evidence remain egui-specific rather than being prematurely promoted into the cross-backend model.

The next pressure is narrower now: **authored objects across multiple layers/windows**, reset/removal behavior, mapping layer-local bindings to flattened renderer order without unstable assumptions, authored-object diffs across frames, and real agent debugging loops against the showcase.

## Non-goals for the first phase

ViewWitness is not currently trying to become:

- a universal GUI standard;
- a replacement for AccessKit or platform accessibility APIs;
- a pixel-perfect rendering format;
- a browser DOM representation;
- a cross-language SDK ecosystem;
- an autonomous GUI agent.

Those may become integration surfaces later. First we want one very good answer for Rust + egui.
