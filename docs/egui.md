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

`witness_from_egui_tree_update` is specifically an **egui adapter**, not a general incremental AccessKit consumer. egui currently provides a complete tree for each generated accessibility frame, allowing ViewWitness to reconstruct parentage from that frame alone.

### Paint-submission evidence

AccessKit does not exhaust rendered reality. egui's public `FullOutput::shapes` exposes the flattened list of `ClippedShape` submissions heading toward the renderer.

ViewWitness currently records provisional `EguiPaintObservation` values containing:

- flattened paint order;
- a compact `EguiPaintKind`;
- visual bounding rectangle;
- finite clip/scissor rectangle when one exists.

The type can deterministically derive bounding-box clip survival:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

This is **not** exact raster/alpha coverage. Likewise, later paint order plus overlapping rectangles does not prove semantic occlusion. Transparency, strokes, meshes, callbacks, and many-to-many widget/paint relationships make that stronger claim unsafe.

Paint evidence therefore remains egui-specific instead of being prematurely promoted into canonical `Witness`.

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
- provisional paint observations.

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

## Exact same-pass semantic + paint capture

Independent semantic and paint streams have different clocks and must not be joined by guesswork. ViewWitness therefore has its own on-demand shared capture point.

`EguiFrameProbe` installs an egui plugin and enables AccessKit. A worker requests one capture. The request only wakes egui. On the next handled output hook, the plugin copies cheap evidence from that exact pass:

- AccessKit update;
- egui-observed viewport rectangle;
- viewport identity;
- cumulative pass number;
- pixels-per-point scale;
- renderer-facing paint observations.

The raw `EguiFrameEvidence` is sent through a bounded channel. Conversion to the canonical semantic `Witness` happens off the GUI thread, producing `EguiCorrelatedCapture`.

The correlated product therefore contains:

- a canonical semantic `Witness`;
- provisional egui paint evidence;
- an explicit request ID;
- egui viewport ID;
- egui cumulative pass number;
- the full observed viewport rectangle.

Its metadata records `semantic_paint_correlation: same_full_output`. That is a stronger claim than anything available by independently reading `egui_inspection` and the continuous paint stream.

### Viewport evidence

The exact correlated path does **not** treat AccessKit root bounds as a viewport surrogate.

Executable testing demonstrated a legitimate headless egui frame whose semantic root had no usable bounds while egui itself still had trustworthy viewport geometry. `EguiFrameProbe` therefore copies `InputState::viewport_rect()` from the same output-hook invocation and rejects invalid/non-finite geometry instead of guessing.

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

`EguiCaptureObserver` is a read-only external client. End-to-end loopback tests prove that an external observer can block on one exact request while the main egui loop continues servicing passes and then receive semantic + paint evidence from one exact `FullOutput`.

A request that cannot obtain a pass returns `TimedOut`; an unrelated protocol handshake is rejected.

## Agent projection and CLI

`EguiCorrelatedCapture` remains the structured, serializable evidence product. Agent text is only a deterministic projection of it.

`correlated_capture_to_agent_text` emits:

1. a correlation header naming request, viewport, pass, viewport rectangle, paint count, and `same_full_output` basis;
2. the normal canonical semantic witness projection;
3. one ordered line per paint submission, including bounds, clip evidence, and derived bounding-box visible fraction.

The projection deliberately does **not** invent a semantic-node ↔ paint-shape identity mapping.

The unified CLI exposes exact capture with:

```text
viewwitness capture-exact [address] [--agent|--yaml] [--derive]
```

The default address is `127.0.0.1:5721`.

- `--agent` prints the correlated agent projection;
- `--yaml` serializes the complete correlated envelope, not merely the semantic witness;
- `--derive` enriches deterministic geometry relations on the semantic `Witness` while leaving observed paint evidence untouched.

## Upstream inspection observer

The optional `observer` feature provides `InspectionObserver`, which speaks the versioned `egui_inspection` protocol directly without requiring MCP.

It currently supports:

- `capture()` — external AccessKit tree → canonical `Witness`;
- `settle()` / `settle_and_capture()` — advance toward idle while preserving `settled: false` as evidence;
- `screenshot()` — PNG raster evidence.

The upstream inspection clock is plugin-owned and differs from egui's viewport-aware cumulative pass number. ViewWitness never joins inspection `step` to `EguiPaintFrame.pass_nr` by equality or assumed offset.

The upstream protocol can also support input operations even though ViewWitness's observer API is intentionally read-only. Keep it loopback-only unless remote exposure is explicitly secured.

## Showcase as a live integration target

The native eframe showcase is now wired as a real ViewWitness pressure target.

At application startup it binds the two ViewWitness loopback listeners before normal UI execution. Through eframe's `CreationContext`, it installs the lightweight paint reporter and exact frame probe. The blocking server loops run on named worker threads.

The resulting live surfaces are:

```text
127.0.0.1:5719  optional upstream egui_inspection semantic/raster endpoint
127.0.0.1:5720  ViewWitness continuous paint stream
127.0.0.1:5721  ViewWitness exact correlated capture request/response
```

This establishes the intended integration pattern for other egui applications:

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

No listener accept loop, socket I/O, serialization, or capture waiting belongs on the render/UI thread.

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

Request an exact correlated semantic + paint capture from another shell:

```powershell
cargo run --features egui --bin viewwitness -- capture-exact
```

Request full YAML instead of the compact agent projection:

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

egui internally has richer widget/layout data than AccessKit alone, and custom-painted canvas objects may have no semantic identity at all.

Even with exact same-pass capture, ViewWitness still refuses to infer that semantic node X produced paint shape Y merely because labels or rectangles happen to coincide.

A promising next path is **explicit application-authored paint identity**. egui's `Painter::add` returns a `ShapeIdx`, while a `Painter` exposes its `LayerId` and clip rectangle. That gives ViewWitness a way to let a custom application bind an authored object identity to the actual paint submission handle at creation time instead of reconstructing identity afterward.

This should remain provisional egui evidence until examples establish a backend-neutral concept. The semantics of such an object must also distinguish:

- application-declared identity/name/role;
- observed paint-handle binding;
- observed/derived bounds and clipping;
- any later mapping from layer-local handle to flattened renderer order.

The showcase custom canvas is the first pressure case for this work.

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
        | one output hook copies semantic + viewport + paint
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

MCP is not the ViewWitness data model. `egui_inspection` is not the ViewWitness data model. AccessKit is not the ViewWitness data model. Paint observations are not the ViewWitness data model. Screenshots are not the ViewWitness data model.

They are evidence sources and integration surfaces around the canonical `Witness` representation.

## Next pressure points

The next egui work should be driven by executable showcase cases:

1. establish the smallest explicit custom-paint annotation API that can bind an application-authored object to the actual `LayerId` + `ShapeIdx` returned by egui;
2. determine whether that layer-local handle can be safely resolved to flattened `FullOutput` paint order without depending on unstable internals;
3. test multi-shape authored objects, clipping, replacement via `Painter::set`, and overlapping authored objects;
4. decide whether authored canvas objects belong only in egui-specific correlated evidence or eventually pressure a backend-neutral extension;
5. continue improving application-authored identity for diff matching without hiding heuristic reconciliation;
6. determine what stronger evidence would actually justify canonical clipping or occlusion relations.
