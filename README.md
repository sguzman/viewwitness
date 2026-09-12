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

ViewWitness deliberately refuses to collapse different kinds of GUI testimony merely because they describe the same application.

The current egui slice distinguishes:

- **semantic evidence** — AccessKit roles, hierarchy, labels, values, actions, states, relations, and logical bounds;
- **paint-submission evidence** — renderer-facing egui shapes, flattened paint order, visual bounds, and clip rectangles;
- **derived clip evidence** — bounding-box survival through an observed clip rectangle;
- **authored custom-paint evidence** — application-declared logical-object semantics, optional authored sub-binding identity, and end-of-pass-verified egui paint bindings;
- **raster evidence** — screenshots returned by the inspected application.

A semantic node is not automatically a paint primitive. Rectangle overlap is not automatically occlusion. Application-authored meaning is not automatically observed meaning. Repeating an authored ID is not automatically object grouping. A screenshot is not automatically frame-correlated with a separately captured semantic tree. ViewWitness records those distinctions instead of guessing across them.

## Repository shape

- `src/` — canonical Rust model, serialization, geometry derivation, egui evidence adapters/reporters, exact capture, diffs, and optional live observers.
- `docs/ontology.md` — the small vocabulary ViewWitness commits to.
- `docs/format-v0.md` — draft wire-format contract.
- `docs/geometry.md` — deterministic geometry relation semantics.
- `docs/egui.md` — egui/AccessKit/rendered-evidence architecture and UI-thread boundary.
- `docs/rendered-evidence.md` — executable findings about paint, clipping, identity, and visual epistemics.
- `docs/roadmap.md` — accepted milestones and remaining pressure points.
- `examples/snapshots/` — representative GUI witnesses.
- `examples/transitions/` — before/after witnesses for state-transition and diff work.
- `examples/showcase.rs` — native eframe pressure surface for capture and diagnosis.
- `tests/` — executable checks over corpus, real egui output, identity, diffs, focused inspection, agent verification, backpressure, custom paint, and live protocol framing.

## Current egui slice

The optional `egui` feature translates egui-produced AccessKit output into the canonical ViewWitness model. It preserves semantic hierarchy, roles, names/values, bounds, visibility, focus, selection/toggle state, actions, selected semantic relations, and explicit identity evidence.

Renderer evidence remains egui-specific. `EguiPaintObservation` records compact paint kind, visual bounds, clip rectangle, and flattened paint order. Bounding-box clipping is derived explicitly; stronger claims such as true visual occlusion are not invented from overlap.

`EguiPaintReporter` provides continuous low-cost paint observation through a bounded nonblocking queue. The GUI/output hook performs cheap copying and `try_send` only; queue saturation drops evidence rather than stalling egui. Worker code can stream those frames to an external read-only observer on `127.0.0.1:5720`.

For heavier diagnosis, `EguiFrameProbe` provides **on-demand exact same-pass capture**. Capture eligibility is fixed at pass start. The requested pass supplies AccessKit semantic output, observed viewport geometry/identity, cumulative pass number, scale, generic paint observations, and explicitly authored custom-paint evidence. The raw evidence crosses a bounded channel and is converted/serialized off the GUI thread.

The result is `EguiCorrelatedCapture`: a canonical semantic `Witness` plus provisional egui evidence known to originate from the same requested pass.

### Explicit custom-paint identity

Custom canvas graphics may have no useful AccessKit identity. ViewWitness supports a narrow explicit solution without post-hoc geometry matching.

```text
EguiAuthoredPaintObject
    id / role / name / semantic_evidence
    bindings[]

EguiAuthoredPaintBinding
    optional authored_binding_id
    binding_evidence
    LayerId + ShapeIdx
    verified_at_end_pass
    final kind / bounds / clip
```

The evidence split is intentional:

```text
object id / role / name       = intended object semantics
optional authored_binding_id  = intended sub-part continuity evidence
LayerId + ShapeIdx            = observed exact-pass execution evidence
final kind / bounds / clip    = observed final slot state
```

`EguiPaintAnnotator::paint_object(...)` explicitly groups several paint submissions into one logical object. Within that scope, `add_shape_with_id(...)` / `bind_shape_with_id(...)` optionally give a rendered sub-part a stable authored key such as `outline`, `handle`, `ring`, or `center`. The older unkeyed forms remain valid.

ViewWitness does **not** merge independent objects merely because their authored IDs repeat, and it does not infer sub-binding identity from geometry, paint kind, or vector position.

### What the executable egui probes establish

Tests now prove all of the following:

- two objects with identical overlapping geometry remain distinct by real paint handle;
- replacing an annotated rectangle through `Painter::set` is observed as the final **circle** occupying that slot;
- one logical object can own multiple concrete bindings, including bindings on different egui layers;
- constituent bindings may share geometry while retaining different kind/clip evidence;
- annotations from ordinary unrequested frames do not leak into a later exact request;
- a valid bound slot reset through egui becomes a verified final `Shape::Noop`, while a genuinely unresolvable handle remains `verified_at_end_pass = false`;
- unchanged paint structure can reproduce the same `ShapeIdx`, but inserting unrelated paint earlier can shift that index while the authored object remains unchanged;
- therefore `ShapeIdx` is **frame-local, structure-sensitive execution evidence**, not a cross-frame object key;
- optional authored binding IDs survive reversed paint-submission order even while the underlying `ShapeIdx` changes.

Clip survival belongs to each binding. `visible_bounds()` / `visible_fraction()` derive bounding-box survival from final bounds and clip evidence. ViewWitness deliberately does not synthesize one object-level visibility number when constituent bindings disagree.

## Cross-frame correlated diffs

`EguiCorrelatedDiff` keeps canonical semantic diffing and egui-specific authored-paint diffing separate.

Authored object matching is conservative:

- an object ID is an automatic continuity key only when unique on both sides;
- duplicate object IDs produce explicit ambiguity rather than heuristic matching;
- object-level changes cover object semantics rather than replacing the whole binding collection.

Within a uniquely matched object:

- a non-empty `authored_binding_id` is a continuity key only when unique within that object on both sides;
- uniquely keyed bindings match independent of submission/vector order;
- duplicate authored binding IDs produce explicit sub-binding ambiguity;
- unkeyed bindings retain conservative relative-ordinal matching;
- keyed and unkeyed bindings may coexist;
- binding additions/removals are first-class records;
- material binding changes are **field-granular** (`kind`, `bounds`, clip, layer, verification, evidence);
- pure `ShapeIdx` churn is reported separately as `execution_handle_churn` and does **not** make the diff materially non-empty.

That means an agent can receive `handle.bounds changed` instead of an opaque object-level `bindings changed` blob.

Generic anonymous paint is not naively list-diffed because it lacks durable identity.

## First executable agent-verification slice

A real-egui test exercises the first evidence-level diagnose/fix/verify pressure case.

One authored object has keyed `body` and `handle` bindings. The broken capture puts the handle far away from the body. The fixed capture moves the same keyed handle onto the body edge while also inserting unrelated anonymous paint before the object, deliberately shifting renderer slots.

The accepted correlated diff isolates exactly:

```text
material:     handle.bounds changed
non-material: body ShapeIdx churned
not reported: body material change
not reported: object-level field="bindings"
```

This is the first executable proof that ViewWitness can preserve the application-level change an agent cares about while separating unrelated renderer bookkeeping churn in the same transition.

The native showcase now contains the corresponding **live broken state**. On the Canvas page, the `Misplaced canvas handle` pressure switch displaces only `showcase:painted-rectangle`'s keyed `handle` while leaving its `outline` fixed. The state and the four binding keys are regression-guarded.

## Exact external capture protocol

`run_egui_capture_server` and `EguiCaptureObserver` expose exact correlated capture through a separate read-only request/response endpoint on `127.0.0.1:5721`.

```text
VIEWWITNESS-EGUI-CAPTURE 2
```

Protocol v2 introduced the one-object-with-`bindings[]` envelope. Optional authored binding IDs are an additive field inside that v2 shape: existing v2 captures without the field deserialize as unkeyed bindings, so no protocol bump is required.

## Focused exact inspection

A full `EguiCorrelatedCapture` remains the complete exact envelope. For agent diagnosis, ViewWitness can also project only one authored object and optionally one authored binding through `correlated_capture_authored_focus_to_agent_text`.

The focused projection deliberately does **not** masquerade as a complete witness. Its header retains request/pass/viewport correlation metadata and labels itself:

```text
projection=authored_focus omitted=canonical_semantics,generic_paint
```

It also reports `object_match_count` and `binding_match_count`. Duplicate authored IDs remain multiple visible matches rather than being silently collapsed; missing IDs produce explicit zero-match output. This gives an agent a small “show me this rendered object/part” surface without weakening provenance or inventing uniqueness.

Focused output is agent text only. YAML remains reserved for the full correlated envelope.

## CLI

Ordinary semantic observation remains available through upstream inspection:

```text
cargo run --features observer --bin viewwitness -- capture --settle=8
```

Exact correlated capture:

```text
cargo run --features egui --bin viewwitness -- capture-exact
cargo run --features egui --bin viewwitness -- capture-exact --yaml
```

Focused live diagnosis:

```text
cargo run --features egui --bin viewwitness -- capture-exact --object=showcase:painted-rectangle
cargo run --features egui --bin viewwitness -- capture-exact --object=showcase:painted-rectangle --binding=handle
```

Saved-envelope inspection:

```text
cargo run --features egui --bin viewwitness -- inspect-exact before.yaml
cargo run --features egui --bin viewwitness -- inspect-exact before.yaml --object=showcase:painted-rectangle --binding=handle
```

Exact before/after verification over saved correlated envelopes:

```text
cargo run --features egui --bin viewwitness -- capture-exact --yaml > before.yaml
# change or interact with the application
cargo run --features egui --bin viewwitness -- capture-exact --yaml > after.yaml
cargo run --features egui --bin viewwitness -- diff-exact before.yaml after.yaml
```

`diff-exact --yaml` emits the structured correlated diff. Agent text is the default and explicitly labels object-level changes, binding additions/removals/field changes, ambiguity, and non-material execution-handle churn.

For focused exact inspection, `--binding` requires `--object`. Focus cannot be combined with `--yaml` or `--derive` because the focused projection intentionally omits canonical semantics and generic paint.

The exact capture command defaults to `127.0.0.1:5721`; a different address may be supplied positionally.

Run the living native pressure surface with:

```text
cargo run --example showcase --features showcase
```

Blocking networking remains on worker threads. The Canvas page deliberately mixes explicitly authored objects with anonymous background/text paint so instrumentation does not pretend to semanticize the whole renderer. Its four authored sub-parts are keyed and regression-guarded:

```text
showcase:painted-rectangle -> outline, handle
showcase:painted-circle    -> ring, center
```

## Two live tempos

The architecture intentionally has two observation tempos:

1. **continuous monitoring** — cheap generic paint metadata through a bounded queue; dropping evidence is preferable to render-thread backpressure;
2. **exact diagnosis** — a requested pass copies semantic + viewport + paint evidence and optional authored identity, then conversion/network/diff work happens off-thread.

The external `egui_inspection` semantic stream and ViewWitness continuous paint stream also use different clocks. ViewWitness never joins them by equality or an assumed fixed offset. Exact capture avoids that reconciliation problem by taking all correlated evidence from one requested egui pass.

## Status

ViewWitness has moved well beyond format-only exploration. The current project has:

- executable synthetic snapshot and transition corpora;
- deterministic geometry derivation;
- first-class canonical `WitnessDiff`;
- real egui/AccessKit semantic capture;
- explicit node identity provenance/stability;
- generic renderer-facing paint evidence and clip-survival derivation;
- bounded nonblocking continuous paint reporting;
- exact same-pass `EguiCorrelatedCapture` and protocol v2;
- explicit authored logical objects with one or many verified layer-local paint bindings;
- optional authored sub-binding IDs for stable cross-frame part continuity;
- executable reset/noop versus missing-handle semantics;
- real cross-frame proof that `ShapeIdx` is structure-sensitive and frame-local;
- identity-aware `EguiCorrelatedDiff` with explicit object and binding ambiguity;
- first-class binding add/remove and field-level material changes;
- deterministic correlated capture/diff agent text;
- full and focused exact inspection through `capture-exact` / `inspect-exact`;
- `diff-exact` over saved correlated envelopes;
- a living eframe showcase with worker-hosted `:5720` and `:5721` services, four guarded keyed Canvas bindings, and a guarded live misplaced-handle defect;
- a real-egui agent-verification pressure test that isolates a keyed handle fix from unrelated renderer-slot churn;
- external semantic/raster observation through `egui_inspection`;
- CI over default and all-features builds.

The v0 canonical `Witness` schema is still intentionally provisional. Generic paint and authored-object/binding evidence remain egui-specific rather than being prematurely promoted into the cross-backend model.

The next pressure is narrower now: **perform an actual source edit against the existing live misplaced-handle scenario, rebuild/recapture, and prove the fix through full correlated diff evidence**. Turning the pressure switch off is useful action/state verification but does not count as source-edit proof. Other open pressure includes safe mapping (if any) from layer-local authored bindings to flattened renderer order, more multi-window/viewport pressure, and stronger raster evidence before any canonical occlusion claim.

## Non-goals for the first phase

ViewWitness is not currently trying to become a universal GUI standard, replace AccessKit/platform accessibility APIs, become a pixel-perfect rendering format, model the browser DOM, or become an autonomous GUI agent.

First we want one very good answer for Rust + egui.
