# egui integration

ViewWitness initially targets Rust + egui. The integration has deliberately distinct evidence layers and two different live tempos. They are related, but they are not interchangeable.

The central rule remains: **egui's GUI/output path may copy cheap evidence and emit bounded work; it must not perform serialization, networking, diffing, searching, persistence, or other heavy work.**

## Evidence layers

### Semantic evidence

egui already produces AccessKit output. ViewWitness consumes that output as observed semantic evidence rather than inventing a competing widget vocabulary at capture time.

The canonical egui adapter preserves semantic roles, names/values, hierarchy, logical bounds, visibility/state, focus/selection/toggle state, actions, identity provenance, selected AccessKit properties, and selected semantic relations.

AccessKit IDs are projected as `ak:<u64>` in v0. Automatically generated identity is treated as `structure_sensitive`: executable probes show it can survive ordinary state changes while changing after structural insertion. Application-authored AccessKit IDs remain additional evidence rather than silently replacing the observed source ID.

### Generic paint-submission evidence

AccessKit does not exhaust rendered reality. egui's public `FullOutput::shapes` exposes the flattened `ClippedShape` submissions heading toward the renderer.

ViewWitness records provisional `EguiPaintObservation` values containing flattened paint order, compact `EguiPaintKind`, visual bounding rectangle, and finite clip/scissor rectangle when one exists.

It can derive:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

This is bounding-box clip evidence, **not** exact raster/alpha coverage. Paint order plus rectangle overlap also does not prove semantic occlusion.

### Explicit authored custom-paint evidence

Custom canvas objects may not exist meaningfully in AccessKit. The authored layer deliberately separates logical-object semantics from concrete renderer bindings:

```rust
EguiAuthoredPaintObject {
    id,
    role,
    name,
    semantic_evidence,
    bindings: Vec<EguiAuthoredPaintBinding>,
}

EguiAuthoredPaintBinding {
    binding_evidence,
    layer_order,
    layer_id,
    shape_index,
    verified_at_end_pass,
    kind,
    bounds,
    clip_rect,
}
```

The epistemic split is first-class:

```text
object id / role / name                semantic_evidence = intended
one concrete LayerId + ShapeIdx slot   binding_evidence = observed
```

`EguiPaintAnnotator::paint_object(...)` is the explicit grouping operation. Its callback receives an `EguiPaintObjectScope`; every `add_shape` / `bind_shape` performed through that scope becomes one concrete binding of that logical object. The singular `EguiPaintAnnotator::add_shape(...)` remains a convenience for an object with one binding.

This prevents a dangerous shortcut: ViewWitness does **not** group independent annotations merely because they repeat the same object ID or overlap geometrically. A dedicated executable test submits two independently annotated objects with the same authored ID and proves they remain two object records.

At `Plugin::on_end_pass`, after application UI code has painted but before egui drains graphic layers into `FullOutput`, the exact frame probe verifies each real layer-local slot. Every binding preserves final verification state, final shape kind, final visual bounds, and final finite clip rectangle.

Executable tests establish:

- two authored objects with identical overlapping bounds remain distinct because they occupy different paint slots;
- an annotated rectangle replaced through `Painter::set` is observed at end-of-pass as the final **circle** in that same slot;
- one explicitly grouped logical object can own several different paint handles;
- two bindings of one object can have identical final visual bounds while carrying different kinds and clipping;
- annotations made on an ordinary unrequested frame do not leak into a later exact capture.

Clip visibility belongs to the **binding**. `EguiAuthoredPaintBinding::visible_bounds()` and `visible_fraction()` use each binding's final verified bounds and clip. The multi-shape pressure test creates one logical object whose two bindings both have 40×40 final bounds: one survives fully, while the second is clipped to a 20×40 half and derives `0.5` visibility.

ViewWitness deliberately does not synthesize a single object-level `visible_fraction` from those two facts. Such an aggregate would require semantics about whether the bindings are additive, decorative, mutually covering, or otherwise related.

This authored evidence remains egui-specific. It is not automatically a canonical `WitnessNode` and it does not create an AccessKit-node ↔ paint-shape mapping.

### Raster evidence

The upstream `egui_inspection` protocol can return screenshots as PNG bytes plus dimensions. Its screenshot response currently lacks a trustworthy shared frame token with the semantic tree, so ViewWitness keeps raster evidence separate rather than claiming exact semantic↔raster correlation.

## Continuous paint monitoring

`EguiPaintReporter` installs at egui's `Plugin::output_hook` boundary. It copies compact renderer evidence into a bounded standard-library channel with `try_send`.

The reporter performs no serialization, networking, persistence, diffing, searching, or inference. If the consumer falls behind, evidence is dropped rather than blocking egui. Executable backpressure tests make this a hard project invariant.

`run_egui_paint_server` moves transport work to a blocking worker thread. The paint stream uses `127.0.0.1:5720`, a ViewWitness-specific versioned handshake, bounded newline-delimited JSON frames, and latest-frame retention for a later observer.

Explicit authored-object bookkeeping is **not** collected continuously. Ordinary annotated frames paint normally but do not retain binding bookkeeping when no exact request is active. Rich identity is therefore an exact-diagnosis cost, not a permanent render-loop tax.

## Exact same-pass capture

Independent semantic and paint streams have different clocks and must not be joined by guesswork. `EguiFrameProbe` provides ViewWitness's on-demand shared capture point.

A worker requests one capture and wakes egui. Capture eligibility is fixed at `on_begin_pass`; a request arriving midway through a pass waits for the next pass rather than collecting only a suffix of authored annotations.

During a requested pass:

1. `on_begin_pass` activates exact capture and authored-paint bookkeeping;
2. normal application UI code runs and paints;
3. `on_end_pass` resolves every explicitly grouped `(LayerId, ShapeIdx)` binding against final layer-local paint lists;
4. `output_hook` copies AccessKit, viewport, generic flattened paint, and resolved authored-object/binding evidence;
5. a bounded channel hands the evidence to worker code.

Worker-side conversion produces `EguiCorrelatedCapture`, containing the canonical semantic `Witness`, provisional generic paint, provisional authored custom-paint evidence, request ID, viewport ID, pass number, and full observed viewport rectangle.

Its metadata records `semantic_paint_correlation: same_full_output`. When authored objects exist it additionally names the evidence basis as application semantics plus verified egui paint handles.

### Viewport evidence

The exact correlated path does **not** treat AccessKit root bounds as a viewport surrogate.

Executable testing demonstrated a valid headless egui frame whose semantic root had no usable bounds while egui itself still had trustworthy viewport geometry. `EguiFrameProbe` therefore copies `InputState::viewport_rect()` and rejects invalid/non-finite geometry instead of guessing.

The older external `InspectionObserver` path still has only the upstream AccessKit tree and may need root-bounds reconstruction. That limitation belongs to that transport, not the canonical model.

## Exact external capture transport

Exact correlated evidence is available outside the application process through ViewWitness's request/response protocol on `127.0.0.1:5721`.

The current handshake is:

```text
VIEWWITNESS-EGUI-CAPTURE 2
```

Protocol v2 was introduced because the serialized authored-paint envelope changed materially: one logical object now owns an explicit `bindings[]` collection instead of duplicating object semantics once per paint handle. A v2 observer rejects a v1 peer rather than silently interpreting an incompatible JSON shape.

`run_egui_capture_server` is blocking worker code. It owns the worker-side `EguiFrameProbe`, requests repaint/capture, waits off-thread, converts off-thread, and serializes off-thread.

`EguiCaptureObserver` is a read-only external client. Loopback tests prove an external observer can wait for one exact request while the main egui loop keeps servicing passes and then receive the complete correlated envelope. Timeouts, alien handshakes, and incompatible older protocol versions remain explicit errors.

## Agent projection and CLI

`EguiCorrelatedCapture` remains the structured evidence product. Agent text is only a deterministic projection.

`correlated_capture_to_agent_text` emits:

1. correlation metadata including generic paint count, authored-object count, and authored-binding count;
2. the canonical semantic witness projection;
3. one `authored-object` line per logical object, with intended semantic provenance and binding count;
4. one `authored-binding` line per concrete binding, carrying observed handle provenance, final bounds/clip/kind, and derived bounding-box visibility when available;
5. one ordered line per generic paint submission.

Object and binding indices are included in the textual projection. This is deliberate: duplicate authored IDs remain visible as duplicate IDs on separate object records instead of being merged implicitly.

For both authored bindings and generic paint, derived clip visibility is labeled `visible_fraction_evidence=derived_bbox_clip`.

The projection deliberately does **not** invent an AccessKit-node ↔ paint-shape identity mapping.

The unified CLI exposes:

```text
viewwitness capture-exact [address] [--agent|--yaml] [--derive]
```

- `--agent` prints all correlated layers without collapsing them;
- `--yaml` serializes the complete envelope, including authored objects and their `bindings[]`;
- `--derive` enriches deterministic geometry relations only on the canonical semantic `Witness`.

## Showcase as a live integration target

The native eframe showcase is a real ViewWitness pressure target.

At startup it binds ViewWitness loopback listeners before normal UI execution. Through eframe's `CreationContext`, it installs the cheap paint reporter and exact frame probe. Blocking server loops run on named worker threads.

```text
127.0.0.1:5719  optional upstream egui_inspection semantic/raster endpoint
127.0.0.1:5720  ViewWitness continuous generic paint stream
127.0.0.1:5721  ViewWitness exact correlated capture request/response
```

The Canvas page now contains two explicit logical authored objects and four concrete paint bindings:

```text
showcase:painted-rectangle
    - outline binding
    - small handle binding

showcase:painted-circle
    - outer ring binding
    - center marker binding
```

The canvas background and text labels remain generic anonymous paint. Exact capture should therefore report two authored objects / four authored bindings plus unrelated generic renderer evidence.

Instrumentation grants identity only where there is an explicit source for that claim.

## Operator examples

Run the showcase:

```powershell
cargo run --example showcase --features showcase
```

Optionally enable upstream inspection too:

```powershell
$env:EGUI_INSPECTION="1"
cargo run --example showcase --features showcase
```

Request an exact correlated capture:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact
```

Request full YAML:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact --yaml
```

Use upstream semantic/raster observation separately:

```powershell
cargo run --features observer --bin viewwitness -- capture --settle=8
cargo run --features observer --bin viewwitness -- screenshot witness.png --scale=1
```

## The remaining widget-to-paint gap

The custom-canvas case is explicitly solvable when the application authors identity. The generic widget-to-paint problem remains open.

ViewWitness still refuses to infer that AccessKit node X produced paint shape Y merely because labels, rectangles, or capture time coincide.

The current proven boundary is:

```text
explicit custom logical object -> one or many paint bindings    proven
binding -> concrete LayerId + ShapeIdx                           proven
LayerId + ShapeIdx -> final layer-local shape                    proven at on_end_pass
final binding bounds + clip -> bbox visibility                   proven derived evidence
arbitrary AccessKit node -> paint binding                        not proven
layer-local binding -> flattened FullOutput global order         not yet proven generally
```

## Current architecture

```text
                                      optional semantic / raster
running egui/eframe app ----------------------------------------> egui_inspection :5719
        |
        | output hook: cheap copy + try_send only
        v
EguiPaintReporter
        |
        | bounded queue
        v
worker: run_egui_paint_server ---------------------------------> :5720 continuous paint

running egui/eframe app
        |
        | explicit capture request wakes repaint
        | on_begin_pass activates requested capture
        | UI explicitly groups logical object -> paint bindings
        | on_end_pass verifies every LayerId + ShapeIdx binding
        | output_hook copies semantic + viewport + generic paint + authored evidence
        v
EguiFrameProbe
        |
        | bounded exact response
        v
worker: run_egui_capture_server -------------------------------> :5721 protocol v2
                                                                      |
                                                                      v
                                                           EguiCaptureObserver
                                                                      |
                                                                      +--> EguiCorrelatedCapture
                                                                      +--> agent projection
                                                                      +--> full YAML
```

## Agent boundary

MCP is not the ViewWitness data model. `egui_inspection` is not the ViewWitness data model. AccessKit is not the ViewWitness data model. Generic paint observations are not the ViewWitness data model. Authored egui paint objects/bindings are not the canonical ViewWitness data model. Screenshots are not the ViewWitness data model.

They are evidence sources and integration surfaces around the canonical `Witness` representation.

## Next pressure points

The next egui work should stay example-driven:

1. test one authored logical object whose bindings live on **different egui layers**, without assuming flattened order;
2. test reset/removal semantics for bound handles;
3. determine whether layer-local handles can be mapped safely to flattened `FullOutput` order without relying on unstable or incomplete assumptions;
4. make authored-object transitions/diffs useful across exact captures while preserving authored-vs-observed provenance;
5. run a real agent debugging loop against the showcase: inspect → identify defect → modify → recapture → verify;
6. decide whether repeated cross-backend pressure ever justifies promoting a generic “authored visual object” concept beyond the egui-specific correlated envelope;
7. continue refusing canonical occlusion until stronger evidence than rectangle overlap exists.
