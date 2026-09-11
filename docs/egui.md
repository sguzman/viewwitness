# egui integration

ViewWitness initially targets Rust + egui. The integration now has several deliberately distinct evidence layers and two different live tempos. They are related, but they are not interchangeable.

The central rule remains: **egui's GUI/output path may copy cheap evidence and emit bounded work; it must not perform serialization, networking, diffing, searching, persistence, or other heavy work.**

## Evidence layers

### Semantic evidence

egui already produces AccessKit output. ViewWitness consumes that output as observed semantic evidence rather than inventing a competing widget vocabulary at capture time.

The canonical egui adapter preserves semantic roles, names/values, hierarchy, logical bounds, visibility/state, focus/selection/toggle state, actions, identity provenance, selected AccessKit properties, and selected semantic relations.

AccessKit IDs are projected as `ak:<u64>` in v0. Automatically generated identity is treated as `structure_sensitive`: executable probes show it can survive ordinary state changes while changing after structural insertion. Application-authored AccessKit IDs remain additional evidence rather than silently replacing the observed source ID.

### Generic paint-submission evidence

AccessKit does not exhaust rendered reality. egui's public `FullOutput::shapes` exposes the flattened `ClippedShape` submissions heading toward the renderer.

ViewWitness records provisional `EguiPaintObservation` values containing:

- flattened paint order;
- compact `EguiPaintKind`;
- visual bounding rectangle;
- finite clip/scissor rectangle when one exists.

It can derive:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

This is bounding-box clip evidence, **not** exact raster/alpha coverage. Paint order plus rectangle overlap also does not prove semantic occlusion.

### Explicit authored custom-paint evidence

Custom canvas objects may not exist meaningfully in AccessKit. `EguiPaintAnnotator` lets the application explicitly bind authored object semantics to the real paint handle returned by egui:

- authored object ID;
- authored role;
- optional authored name;
- painter `LayerId`;
- returned layer-local `ShapeIdx`.

The epistemic split is first-class:

```text
id / role / name             semantic_evidence = intended
LayerId + ShapeIdx binding   binding_evidence = observed
```

At `Plugin::on_end_pass`, after application UI code has painted but before egui drains graphic layers into `FullOutput`, the exact frame probe verifies the real layer-local slot. `EguiAuthoredPaintObject` preserves whether verification succeeded plus the final shape kind, final visual bounds, and final finite clip rectangle.

Executable tests establish that this is real handle identity rather than geometry matching:

- two authored objects with identical overlapping bounds remain distinct because they occupy different paint slots;
- an annotated rectangle replaced through `Painter::set` is observed at end-of-pass as the final **circle** in that same slot;
- annotations made on an ordinary unrequested frame do not leak into a later exact capture.

Authored objects also compose with clipping. `EguiAuthoredPaintObject::visible_bounds()` and `visible_fraction()` derive clip survival from the **final verified handle evidence**. A test uses a 40×40 authored rectangle under a finite clip preserving a 20×40 half and proves a derived visible fraction of `0.5`.

That fraction has the same narrow meaning as generic paint visibility: half of the axis-aligned visual bounding rectangle survives the clip. It does not claim half the painted pixels or alpha survive.

This authored evidence remains egui-specific. It is not automatically a canonical `WitnessNode` and it does not create an AccessKit-node ↔ paint-shape mapping.

### Raster evidence

The upstream `egui_inspection` protocol can return screenshots as PNG bytes plus dimensions. Its screenshot response currently lacks a trustworthy shared frame token with the semantic tree, so ViewWitness keeps raster evidence separate rather than claiming exact semantic↔raster correlation.

## Continuous paint monitoring

`EguiPaintReporter` installs at egui's `Plugin::output_hook` boundary. It copies compact renderer evidence into a bounded standard-library channel with `try_send`.

`EguiPaintFrame` carries viewport identity, cumulative pass number, pixels per point, dropped-frame evidence, and generic paint observations.

The reporter performs no serialization, networking, persistence, diffing, searching, or inference. If the consumer falls behind, evidence is dropped rather than blocking egui. Executable backpressure tests make this a hard project invariant.

`run_egui_paint_server` moves transport work to a blocking worker thread. The v0 paint stream uses `127.0.0.1:5720`, a ViewWitness-specific versioned handshake, bounded newline-delimited JSON frames, and latest-frame retention for a later observer.

Explicit authored-object bookkeeping is **not** collected continuously. An annotated paint call on an ordinary unrequested frame pays an atomic request-state read but does not push object bookkeeping into the pending annotation buffer. Rich identity is therefore an exact-diagnosis cost, not a permanent render-loop tax.

## Exact same-pass capture

Independent semantic and paint streams have different clocks and must not be joined by guesswork. `EguiFrameProbe` provides ViewWitness's on-demand shared capture point.

A worker requests one capture and wakes egui. Capture eligibility is fixed at `on_begin_pass`; a request arriving midway through a pass waits for the next pass rather than collecting only a suffix of authored annotations.

During a requested pass:

1. `on_begin_pass` activates exact capture and authored-paint bookkeeping;
2. normal application UI code runs and paints;
3. `on_end_pass` resolves authored `(LayerId, ShapeIdx)` handles against final layer-local paint lists;
4. `output_hook` copies AccessKit, viewport, generic flattened paint, and resolved authored-object evidence;
5. a bounded channel hands the evidence to worker code.

The exact `EguiFrameEvidence` can therefore contain:

- AccessKit update;
- egui-observed viewport rectangle;
- viewport identity;
- cumulative pass number;
- pixels-per-point scale;
- generic renderer-facing paint observations;
- explicit authored custom-paint bindings from the same requested pass.

Worker-side conversion produces `EguiCorrelatedCapture`, which contains the canonical semantic `Witness`, provisional generic paint, provisional authored custom-paint evidence, request ID, viewport ID, pass number, and full observed viewport rectangle.

Its metadata records `semantic_paint_correlation: same_full_output`. When authored objects exist it also records the application-semantics + verified-paint-handle evidence basis.

### Viewport evidence

The exact correlated path does **not** treat AccessKit root bounds as a viewport surrogate.

Executable testing demonstrated a valid headless egui frame whose semantic root had no usable bounds while egui itself still had trustworthy viewport geometry. `EguiFrameProbe` therefore copies `InputState::viewport_rect()` and rejects invalid/non-finite geometry instead of guessing.

The older external `InspectionObserver` path still has only the upstream AccessKit tree and may need root-bounds reconstruction. That limitation belongs to that transport, not the canonical model.

## Exact external capture transport

Exact correlated evidence is available outside the application process through a ViewWitness-owned request/response protocol on `127.0.0.1:5721`.

`run_egui_capture_server` is blocking worker code. It owns the worker-side `EguiFrameProbe`, requests repaint/capture, waits off-thread, converts off-thread, and serializes off-thread.

`EguiCaptureObserver` is a read-only external client. Loopback tests prove an external observer can wait for one exact request while the main egui loop keeps servicing passes and then receive the complete correlated envelope. Timeouts and alien handshakes remain explicit errors.

## Agent projection and CLI

`EguiCorrelatedCapture` remains the structured evidence product. Agent text is only a deterministic projection.

`correlated_capture_to_agent_text` emits:

1. correlation metadata including generic paint and authored-object counts;
2. the canonical semantic witness projection;
3. one line per explicit authored object with intended semantic provenance, observed handle-binding provenance, final bounds/clip, and derived bounding-box visibility when available;
4. one ordered line per generic paint submission.

For both authored and generic paint, derived clip visibility is labeled `visible_fraction_evidence=derived_bbox_clip`.

The projection deliberately does **not** invent an AccessKit-node ↔ paint-shape identity mapping.

The unified CLI exposes:

```text
viewwitness capture-exact [address] [--agent|--yaml] [--derive]
```

- `--agent` prints all correlated layers without collapsing them;
- `--yaml` serializes the complete envelope, including authored objects;
- `--derive` enriches deterministic geometry relations only on the canonical semantic `Witness`.

## Upstream inspection observer

The optional `observer` feature provides `InspectionObserver`, which speaks the versioned `egui_inspection` protocol directly without requiring MCP.

It supports semantic capture, settle/settle-and-capture, and PNG screenshot retrieval. Its plugin-owned inspection step is not equated with egui's viewport-aware cumulative pass number.

The upstream protocol supports more than ViewWitness exposes, so keep it loopback-only unless remote exposure is explicitly secured.

## Showcase as a live integration target

The native eframe showcase is a real ViewWitness pressure target.

At startup it binds ViewWitness loopback listeners before normal UI execution. Through eframe's `CreationContext`, it installs the cheap paint reporter and exact frame probe. Blocking server loops run on named worker threads.

```text
127.0.0.1:5719  optional upstream egui_inspection semantic/raster endpoint
127.0.0.1:5720  ViewWitness continuous generic paint stream
127.0.0.1:5721  ViewWitness exact correlated capture request/response
```

The Canvas page explicitly annotates its painted rectangle and circle with stable authored IDs and roles. The canvas background and text labels remain generic anonymous paint. Exact capture therefore demonstrates a mixture of ordinary AccessKit semantics, generic paint submissions, and selectively identified custom-painted objects.

Instrumentation grants identity only where there is an explicit source for that claim.

The integration pattern for other egui apps is:

```text
bind/start worker infrastructure before ordinary UI work
        |
        v
install cheap egui plugins during application creation
        |
        v
GUI pass: copy/emit bounded evidence only
        |
        v
worker threads: wait, convert, derive, serialize, network, persist
```

No listener accept loop, socket I/O, serialization, capture waiting, or expensive analysis belongs on the render/UI thread.

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
explicit custom object -> LayerId + ShapeIdx       proven for instrumented paint
LayerId + ShapeIdx -> final layer-local shape      proven at on_end_pass
final handle bounds + clip -> bbox visibility      proven derived evidence
arbitrary AccessKit node -> paint handle           not proven
all paint handles -> flattened global order        not yet proven generally
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
        | UI may bind authored object -> LayerId + ShapeIdx
        | on_end_pass verifies authored paint handles and final clip
        | output_hook copies semantic + viewport + paint + authored evidence
        v
EguiFrameProbe
        |
        | bounded exact response
        v
worker: run_egui_capture_server -------------------------------> :5721 request/response
                                                                      |
                                                                      v
                                                           EguiCaptureObserver
                                                                      |
                                                                      +--> EguiCorrelatedCapture
                                                                      +--> agent projection
                                                                      +--> full YAML
```

## Agent boundary

MCP is not the ViewWitness data model. `egui_inspection` is not the ViewWitness data model. AccessKit is not the ViewWitness data model. Generic paint observations are not the ViewWitness data model. Authored egui paint objects are not the ViewWitness data model. Screenshots are not the ViewWitness data model.

They are evidence sources and integration surfaces around the canonical `Witness` representation.

## Next pressure points

The next egui work should stay example-driven:

1. pressure one authored logical object composed of **multiple paint handles**;
2. test authored objects across different layers/windows;
3. test reset/removal semantics for bound handles;
4. determine whether layer-local handles can be mapped safely to flattened `FullOutput` order without relying on unstable or incomplete assumptions;
5. make authored-object transitions/diffs useful across exact captures while preserving authored-vs-observed provenance;
6. decide whether repeated cross-backend pressure ever justifies promoting a generic “authored visual object” concept beyond the egui-specific correlated envelope;
7. continue refusing canonical occlusion until stronger evidence than rectangle overlap exists.
