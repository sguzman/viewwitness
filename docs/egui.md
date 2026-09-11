# egui integration

ViewWitness initially targets Rust + egui. The integration now has three deliberately distinct evidence channels: semantic tree evidence, renderer/clip evidence, and raster screenshot evidence. They are related but must not be collapsed into one another.

## Semantic source

egui already produces AccessKit semantic output. ViewWitness consumes that output as observed evidence instead of inventing a second widget-semantic vocabulary at capture time.

The `egui` crate feature exposes:

- `EguiCaptureContext`;
- `witness_from_egui_output`;
- `witness_from_egui_tree_update`;
- provisional `EguiPaintObservation` plus `paint_observations_from_egui_output`;
- `EguiPaintReporter` for bounded nonblocking paint reporting;
- `run_egui_paint_server` and `EguiPaintObserver` for a read-only external paint side channel.

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

## Semantic facts mapped today

The canonical egui adapter preserves:

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

## Renderer/clip evidence

AccessKit does not exhaust rendered reality. egui's public `FullOutput::shapes` provides a second evidence source: the flattened list of `ClippedShape` values passed toward the renderer.

The provisional research type records:

```rust
pub struct EguiPaintObservation {
    pub order: usize,
    pub kind: EguiPaintKind,
    pub bounds: Rect,
    pub clip_rect: Option<Rect>,
}
```

`EguiPaintKind` is a small `Copy` enum with stable snake-case serialization. Fixed paint classification therefore avoids allocating a heap `String` per shape on egui's output path.

`order` follows the flattened renderer-facing shape sequence: smaller values are painted earlier/farther back. `bounds` comes from `Shape::visual_bounding_rect()`. A finite egui clip/scissor rectangle is preserved directly; an effectively unbounded/non-finite clip is represented as `None` rather than converted into invented finite geometry.

The type can deterministically derive:

```text
visible_bounds = bounds ∩ clip_rect
visible_fraction = area(visible_bounds) / area(bounds)
```

The executable paint probe proves that renderer submissions and clipping are distinct facts: a custom shape can remain in `FullOutput::shapes` while being partially clipped or completely clipped to `visible_fraction == 0.0`.

This `visible_fraction` is explicitly a **bounding-box fraction**, not exact alpha/pixel coverage. Paint order plus overlapping rectangles is also not sufficient to claim canonical `occludes`; transparency, strokes, meshes, callbacks, and many-to-many widget/paint relationships make that stronger claim unsafe.

Paint observations therefore remain egui-specific research evidence outside canonical `Witness` for now. See `docs/rendered-evidence.md`.

## Nonblocking live paint reporting

`EguiPaintReporter` installs at egui's public `Plugin::output_hook` boundary. It copies compact renderer evidence into a bounded standard-library channel and calls `try_send` only.

`EguiPaintFrame` carries:

- raw numeric egui viewport identity;
- cumulative egui pass number for that viewport;
- pixels per point;
- `dropped_before`, recording reporter frames lost to bounded-queue saturation since the previous successful delivery;
- provisional paint observations.

The reporter intentionally performs no serialization, persistence, networking, diffing, searching, or model work.

The executable backpressure test fills a one-frame queue, runs another egui pass with no consumer progress, then proves that rendering continues, the saturated frame is dropped, and the next delivered frame reports that loss. This is a hard architectural property rather than an optimization suggestion.

Once the receiver disconnects entirely, the plugin stops collecting paint metadata on future outputs.

## Read-only external paint side channel

`run_egui_paint_server` is a blocking **worker-thread** transport over the reporter receiver. It is ViewWitness-owned and intentionally separate from `egui_inspection`.

The v0 protocol is deliberately narrow:

- default loopback endpoint `127.0.0.1:5720`;
- textual versioned handshake `VIEWWITNESS-EGUI-PAINT 1`;
- compact newline-delimited JSON `EguiPaintFrame` messages;
- 64 MiB defensive message ceiling;
- short network write timeout so a stalled observer is disposable;
- one active observer, with the newest successfully initialized observer replacing the older one;
- latest-frame retention while no observer is connected.

`EguiPaintObserver` validates the handshake and exposes blocking `next_frame()` reads.

The loopback integration test paints a frame **before** an observer connects and proves that the worker retains/replays that frame without another repaint, then proves that later egui passes stream normally. A negative-control test rejects an unrelated protocol handshake.

This side channel is read-only. It does not inject clicks, keys, pointer movement, or any other application mutation.

## The widget-to-paint gap

egui internally tracks rich `WidgetRect` / `WidgetRects` data including widget ID, parent UI ID, layer ID, full rectangle, clipped interaction rectangle, interaction sense, enabled state, and back-to-front ordering within layers.

That is extremely useful evidence for ViewWitness, but the complete table is not currently exposed by the public generic plugin callbacks or the `egui_inspection` protocol. The public plugin `output_hook` can observe `FullOutput`, while `egui_inspection` exports AccessKit trees and screenshot capture rather than the complete widget-rectangle or renderer-shape tables.

ViewWitness must therefore not guess an AccessKit-node ↔ paint-shape relationship from coincident geometry and report it as observation.

## Coordinate caution

AccessKit nodes can carry transforms. v0 records `accesskit_transform_present: true` when one exists, but does not yet compose transforms into canonical ViewWitness bounds. Consumers should therefore treat transformed-node geometry as provisional until transform handling is implemented and tested.

Likewise, AccessKit `clips_children` is preserved as observed semantic evidence, but ViewWitness does not derive node-level clipping from it. The paint probe has honest per-shape clip geometry, but semantic-node ↔ paint-shape linkage is not yet available.

## In-process frame conversion

Tests and custom integrations can enable AccessKit on an `egui::Context`, produce a `FullOutput`, and translate that frame immediately into a canonical semantic `Witness`. The same `FullOutput` can separately be inspected for provisional paint evidence.

This path is currently the only place where semantic and paint evidence can be known to originate from the **same `FullOutput`** without temporal inference.

It is ideal for:

- deterministic unit/integration tests;
- fixture generation;
- small custom applications that explicitly request a witness;
- development of semantic and renderer evidence contracts.

It must remain lightweight on the UI path. Capturing/copying cheap frame evidence is acceptable; expensive serialization, diffing, searching, inference, persistence, or network serving is not.

## Live eframe semantic/raster observation

The optional `observer` feature provides an external `InspectionObserver` that speaks the versioned `egui_inspection` protocol directly. It does not depend on MCP.

The observer currently supports three read-only synchronization/evidence operations:

- `capture()` — request the current AccessKit tree and convert it into a canonical `Witness`;
- `settle()` / `settle_and_capture()` — let the app advance toward idle before observing it, while preserving `settled: false` as evidence rather than failure;
- `screenshot()` — request raster evidence as PNG bytes plus pixel dimensions.

For semantic capture, the observer derives viewport dimensions from observed root bounds. If the root does not provide usable bounds, it returns an error instead of inventing viewport geometry.

Loopback mock-peer integration tests exercise the real TCP handshake and MessagePack framing for tree capture, settle sequencing, and screenshot transport in CI without requiring a graphical desktop session.

### Raster evidence is not frame-correlated witness evidence

The current upstream screenshot response contains PNG bytes and dimensions but **does not contain an inspection step/frame number**. Screenshot capture itself also involves an additional frame.

Therefore ViewWitness does not claim that a screenshot and a separately captured semantic `Witness` describe the exact same frame. Screenshots are supporting raster evidence unless a future transport supplies trustworthy temporal correlation.

For the same reason, the CLI exposes screenshot capture as a separate command rather than silently embedding raster data into a witness document.

### Semantic inspection and paint reporting use different clocks

The external semantic and paint channels are also not exactly frame-correlated today.

`egui_inspection` owns a private `step` counter inside its plugin and increments it in that plugin's `output_hook`. `EguiPaintReporter` records `Context::cumulative_pass_nr()` for the active viewport. The first counter is inspection-plugin-global; the second is egui viewport-aware.

ViewWitness therefore **must not join `Witness.capture.frame` to `EguiPaintFrame.pass_nr` by equality or by an assumed fixed offset**.

Both observations remain valid independently. Exact fusion requires a shared ViewWitness capture point, or an upstream protocol that provides a shared trustworthy frame token.

## Operator examples

Run the native showcase with inspection enabled in PowerShell:

```powershell
$env:EGUI_INSPECTION="1"
cargo run --example showcase --features showcase
```

From another shell, semantic capture can use the unified CLI:

```powershell
cargo run --features observer --bin viewwitness -- capture --settle=8
```

Save separate raster evidence at logical-point-like scale:

```powershell
cargo run --features observer --bin viewwitness -- screenshot witness.png --scale=1
```

The inspection endpoint defaults to `127.0.0.1:5719`. Keep inspection bound to loopback unless remote exposure is explicitly secured: the upstream protocol can expose GUI state and also supports input operations, even though ViewWitness's current observer API is intentionally read-only.

The ViewWitness paint side channel reserves `127.0.0.1:5720` by convention. The server is not automatically attached yet; an application must install `EguiPaintReporter` and run `run_egui_paint_server` on a worker thread.

## Current architecture

```text
                                      semantic / raster
running egui/eframe app ----------------------------------> egui_inspection :5719
        |                                                          |
        |                                                          v
        |                                                InspectionObserver
        |                                                          |
        |                                                          +--> canonical Witness
        |                                                          +--> raster PNG
        |
        | output_hook: cheap copy + try_send only
        v
EguiPaintReporter
        |
        | bounded queue
        v
worker: run_egui_paint_server ----------------------------> paint side channel :5720
                                                                  |
                                                                  v
                                                         EguiPaintObserver
                                                                  |
                                                                  +--> EguiPaintFrame
```

The two external streams are intentionally not fused by clock guesswork.

This preserves the core project rule: **the GUI thread reports GUI state; it does not become the worker responsible for analyzing that state.**

## Agent boundary

MCP is not the ViewWitness data model. egui inspection is not the ViewWitness data model. AccessKit is not the ViewWitness data model. Paint observations are not the ViewWitness data model. Screenshots are not the ViewWitness data model.

They are evidence sources and integration surfaces around the canonical `Witness` representation.

Keeping those layers separate lets the egui-first implementation exploit mature tooling without making the ontology or serialized format hostage to any one transport or representation.

## Next pressure points

The next egui work should answer concrete questions through the showcase and evidence channels:

1. what is the cheapest **on-demand shared capture point** that can copy semantic + paint evidence from one `FullOutput` and immediately hand it off-thread;
2. whether shared capture should emit raw AccessKit evidence plus paint or convert the semantic side to canonical `Witness` before handoff;
3. how to trigger a requested shared capture/repaint without turning the reporting plugin into a worker or creating a `Context` ownership cycle;
4. how application-authored identity should improve diff matching without hiding heuristic reconciliation;
5. how custom-painted/canvas objects should acquire explicit semantic identity when AccessKit cannot supply it;
6. whether observed root bounds remain a reliable viewport source across native platforms and multiple viewport configurations;
7. what evidence would be sufficient to promote clipping, paint order, or occlusion concepts into the canonical cross-backend model.
