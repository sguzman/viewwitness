# egui integration

ViewWitness initially targets Rust + egui. The integration now has several deliberately distinct evidence layers and two different live tempos. They are related, but they are not interchangeable.

The central rule remains: **egui's GUI/output path may copy cheap evidence and emit bounded work; it must not perform serialization, networking, diffing, searching, persistence, or other heavy work.**

## Evidence layers

### Semantic evidence

egui already produces AccessKit output. ViewWitness consumes that output as observed semantic evidence rather than inventing a competing widget vocabulary at capture time.

The canonical egui adapter preserves today:

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
- identity provenance/stability plus optional application-authored ID;
- selected AccessKit properties such as busy/read-only/required/modal/expanded;
- semantic relations such as `labels`, `describes`, and `controls`.

AccessKit IDs are projected as `ak:<u64>` in v0. Captured nodes carry identity evidence such as:

```yaml
identity:
  provenance: accesskit_node_id
  stability: structure_sensitive
  author_id: optional-application-id
```

`structure_sensitive` is deliberate. Executable probes show that ordinary state changes can preserve an automatically generated egui/AccessKit identity while structural insertion can change it. Application-authored `author_id` is preserved as additional evidence rather than silently replacing the observed AccessKit node ID.

### Generic paint-submission evidence

AccessKit does not exhaust rendered reality. egui's public `FullOutput::shapes` exposes the flattened list of `ClippedShape` submissions heading toward the renderer.

ViewWitness records provisional `EguiPaintObservation` values containing:

- flattened paint order;
- a compact `EguiPaintKind`;
- visual bounding rectangle;
- finite clip/scissor rectangle when one exists.

The type can deterministically derive bounding-box clip survival:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

This is **not** exact raster/alpha coverage. Later paint order plus overlapping rectangles also does not prove semantic occlusion. Transparency, strokes, meshes, callbacks, and many-to-many object/paint relationships make that stronger claim unsafe.

Generic paint evidence therefore remains egui-specific instead of being prematurely promoted into canonical `Witness`.

### Explicit authored custom-paint evidence

Custom canvas objects may not exist meaningfully in AccessKit. ViewWitness now supports an explicit application-instrumented evidence path rather than trying to discover such objects from coincident geometry.

`EguiPaintAnnotator` binds application-authored object semantics to the real paint handle returned by egui:

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

The object meaning comes from the application. The fact that this application object was bound to that concrete egui paint slot is observed execution evidence. ViewWitness does not blur those two claims merely because they travel together.

At `Plugin::on_end_pass`, after application UI code has painted but before egui drains graphic layers into `FullOutput`, the frame probe verifies the exact layer-local handle against egui's final paint list. `EguiAuthoredPaintObject` can therefore preserve:

- layer category and numeric layer ID;
- shape index;
- whether the handle was verified at end-of-pass;
- final shape kind;
- final visual bounds;
- final finite clip rectangle when available.

Executable tests deliberately use identical overlapping geometry for two authored objects. They remain distinct because they occupy different paint handles. Another test annotates a rectangle and then replaces that exact slot via `Painter::set`; ViewWitness reports the final shape as a **circle**, demonstrating that the binding follows the real egui slot rather than remembered geometry or the initially submitted shape.

This evidence remains provisional and egui-specific. It is not automatically a canonical `WitnessNode` and it does not create an AccessKit-node ↔ paint-shape mapping.

### Raster evidence

The upstream `egui_inspection` protocol can return screenshots as PNG bytes plus dimensions. Raster evidence is useful visual testimony, but its current response does not contain a trustworthy shared frame token with the semantic tree.

ViewWitness therefore keeps screenshots separate from semantic witnesses rather than silently claiming frame correlation that the transport does not prove.

## Continuous paint monitoring

`EguiPaintReporter` installs at egui's `Plugin::output_hook` boundary. It copies compact renderer evidence into a bounded standard-library channel with `try_send`.

`EguiPaintFrame` carries:

- egui viewport identity;
- cumulative egui pass number;
- pixels per point;
- `dropped_before`, recording frames lost because the bounded queue was saturated;
- provisional generic paint observations.

The reporter performs no serialization, networking, persistence, diffing, searching, or inference. If the consumer falls behind, evidence is dropped rather than blocking egui. Executable backpressure tests make this a project invariant rather than a performance suggestion.

`run_egui_paint_server` moves transport work to a blocking worker thread. The v0 paint side channel uses:

- `127.0.0.1:5720` by convention;
- versioned handshake `VIEWWITNESS-EGUI-PAINT 1`;
- newline-delimited compact JSON frames;
- defensive message bounds;
- short network write timeout;
- one active observer;
- latest-frame retention while no observer is connected.

`EguiPaintObserver` is read-only. This channel is intended for cheap, disposable monitoring rather than exact diagnosis.

Explicit authored-object bookkeeping is **not** collected continuously. An annotated paint call on an ordinary unrequested frame pays an atomic request-state read but does not push object bookkeeping into the pending annotation buffer. Rich identity is therefore an exact-diagnosis cost, not a permanent render-loop tax.

## Exact same-pass capture

Independent semantic and paint streams have different clocks and must not be joined by guesswork. ViewWitness therefore has its own on-demand shared capture point.

`EguiFrameProbe` installs an egui plugin and enables AccessKit. A worker requests one capture and wakes egui.

Capture eligibility is fixed at `on_begin_pass`. This prevents a request arriving halfway through a pass from collecting only a suffix of authored paint annotations while still claiming a complete correlated frame.

During a requested pass:

1. `on_begin_pass` activates exact capture and authored-paint bookkeeping;
2. normal application UI code runs and paints;
3. `on_end_pass` resolves any authored `(LayerId, ShapeIdx)` handles against the final layer-local paint lists;
4. `output_hook` copies AccessKit, viewport, generic flattened paint, and the already-resolved authored-object evidence;
5. a bounded channel hands the exact evidence to worker code.

The requested `EguiFrameEvidence` therefore can contain:

- AccessKit update;
- egui-observed viewport rectangle;
- viewport identity;
- cumulative pass number;
- pixels-per-point scale;
- generic renderer-facing paint observations;
- explicit authored custom-paint bindings from the same requested pass.

Conversion to the canonical semantic `Witness` happens off the GUI thread, producing `EguiCorrelatedCapture`.

The correlated product contains:

- a canonical semantic `Witness`;
- provisional generic egui paint evidence;
- provisional authored custom-paint evidence;
- explicit request ID;
- egui viewport ID;
- egui cumulative pass number;
- full observed viewport rectangle.

Its metadata records `semantic_paint_correlation: same_full_output`. When authored objects are present it also records that authored paint evidence consists of application semantics plus verified egui paint-handle binding.

### Viewport evidence

The exact correlated path does **not** treat AccessKit root bounds as a viewport surrogate.

Executable testing demonstrated a legitimate headless egui frame whose semantic root had no usable bounds while egui itself still had trustworthy viewport geometry. `EguiFrameProbe` therefore copies `InputState::viewport_rect()` and rejects invalid/non-finite geometry instead of guessing.

The older external `InspectionObserver` semantic path still has only the upstream AccessKit tree available, so it derives viewport size from observed root bounds and errors when those bounds are unusable. That limitation belongs to that transport, not to the canonical model.

## Exact external capture transport

Exact correlated evidence is available outside the application process through a ViewWitness-owned request/response protocol.

The v0 endpoint uses:

- default loopback address `127.0.0.1:5721`;
- a versioned ViewWitness handshake;
- explicit `CAPTURE` requests;
- bounded JSON responses carrying the complete `EguiCorrelatedCapture`;
- worker-side timeout and structured error propagation.

`run_egui_capture_server` is blocking worker code. It owns the worker-side `EguiFrameProbe`, requests a repaint/capture, waits off-thread, converts off-thread, and serializes off-thread.

`EguiCaptureObserver` is a read-only external client. End-to-end loopback tests prove that an external observer can block on one exact request while the main egui loop continues servicing passes and then receive the complete correlated envelope.

A request that cannot obtain a pass returns `TimedOut`; an unrelated protocol handshake is rejected.

## Agent projection and CLI

`EguiCorrelatedCapture` remains the structured, serializable evidence product. Agent text is only a deterministic projection of it.

`correlated_capture_to_agent_text` emits:

1. a correlation header naming request, viewport, pass, viewport rectangle, generic paint count, authored-object count, and `same_full_output` basis;
2. the normal canonical semantic witness projection;
3. one line per explicit authored object, including intended semantic provenance and observed binding provenance;
4. one ordered line per generic paint submission, including bounds, clip evidence, and derived bounding-box visible fraction.

The projection deliberately does **not** invent an AccessKit semantic-node ↔ paint-shape identity mapping.

The unified CLI exposes exact capture with:

```text
viewwitness capture-exact [address] [--agent|--yaml] [--derive]
```

The default address is `127.0.0.1:5721`.

- `--agent` prints all correlated evidence layers without collapsing them;
- `--yaml` serializes the complete correlated envelope, including authored objects when present;
- `--derive` enriches deterministic geometry relations on the canonical semantic `Witness` while leaving observed egui evidence untouched.

## Upstream inspection observer

The optional `observer` feature provides `InspectionObserver`, which speaks the versioned `egui_inspection` protocol directly without requiring MCP.

It currently supports:

- `capture()` — external AccessKit tree → canonical `Witness`;
- `settle()` / `settle_and_capture()` — advance toward idle while preserving `settled: false` as evidence;
- `screenshot()` — PNG raster evidence.

The upstream inspection clock is plugin-owned and differs from egui's viewport-aware cumulative pass number. ViewWitness never joins inspection `step` to `EguiPaintFrame.pass_nr` by equality or assumed offset.

The upstream protocol can also support input operations even though ViewWitness's observer API is intentionally read-only. Keep it loopback-only unless remote exposure is explicitly secured.

## Showcase as a live integration target

The native eframe showcase is a real ViewWitness pressure target.

At application startup it binds the two ViewWitness loopback listeners before normal UI execution. Through eframe's `CreationContext`, it installs the lightweight paint reporter and exact frame probe. Blocking server loops run on named worker threads.

The live surfaces are:

```text
127.0.0.1:5719  optional upstream egui_inspection semantic/raster endpoint
127.0.0.1:5720  ViewWitness continuous generic paint stream
127.0.0.1:5721  ViewWitness exact correlated capture request/response
```

The Canvas page now pressure-tests explicit authored custom paint. Its painted rectangle and circle use stable authored IDs and roles through `EguiPaintAnnotator`. The canvas background and text labels remain unannotated generic paint. An exact capture should therefore contain a mixture of:

- ordinary AccessKit semantics;
- generic anonymous renderer submissions;
- exactly identified custom-painted objects where the application deliberately supplied identity.

This is the desired epistemic behavior. Instrumentation grants identity only where there is an explicit source for that claim.

The integration pattern for other egui applications is:

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

Run the native showcase:

```powershell
cargo run --example showcase --features showcase
```

If upstream semantic/raster inspection is also wanted:

```powershell
$env:EGUI_INSPECTION="1"
cargo run --example showcase --features showcase
```

Request an exact correlated capture from another shell:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact
```

On the Canvas page, the agent projection can now include lines such as authored objects for `showcase:painted-rectangle` and `showcase:painted-circle` in addition to generic paint submissions.

Request full YAML instead:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact --yaml
```

Use the upstream semantic observer separately:

```powershell
cargo run --features observer --bin viewwitness -- capture --settle=8
```

Save separate raster evidence:

```powershell
cargo run --features observer --bin viewwitness -- screenshot witness.png --scale=1
```

## The remaining widget-to-paint gap

The custom-canvas case has moved from “unknown” to “explicitly solvable when the application authors identity,” but the generic widget-to-paint problem remains open.

ViewWitness still refuses to infer that AccessKit node X produced paint shape Y merely because labels, rectangles, or capture time coincide.

egui internally has richer widget/layout data than AccessKit alone, including `WidgetRect` / `WidgetRects`. The complete collection is not currently exposed through the same generic public capture path used here.

The current authored custom-paint binding also proves only the actual **layer-local paint handle** and the final shape at that slot. ViewWitness does not yet claim a safe flattened `FullOutput` order for every annotated handle across all unusual layer-order/fallback cases.

That distinction matters:

```text
explicit custom object -> LayerId + ShapeIdx       proven for instrumented paint
LayerId + ShapeIdx -> final layer-local shape      proven at on_end_pass
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
        | on_end_pass verifies authored paint handles
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
2. test authored objects under finite clipping and across different layers/windows;
3. test handle reset/replacement semantics beyond the accepted `Painter::set` case;
4. determine whether layer-local handles can be mapped safely to flattened `FullOutput` order without relying on unstable or incomplete assumptions;
5. make authored-object transitions/diffs useful across exact captures while preserving authored-vs-observed provenance;
6. decide whether repeated cross-backend pressure ever justifies promoting a generic “authored visual object” concept beyond the egui-specific correlated envelope;
7. continue refusing canonical occlusion until stronger evidence than rectangle overlap exists.
