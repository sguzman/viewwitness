# ViewWitness roadmap

ViewWitness is a medium-sized infrastructure project: a durable semantic/data contract plus capture, derivation, serialization, diffing, examples, and agent-facing tooling. It is intentionally smaller in ambition than a renderer or general GUI framework.

## M0 — executable model exploration

**Established and continuing as a permanent discipline.**

- canonical Rust witness structs;
- YAML projection;
- lightweight structural validator;
- mini-ontology notes;
- rich synthetic witness corpus;
- transition fixtures with stable IDs;
- CI that parses and validates every fixture.

M0 is no longer a blocking phase, but corpus expansion continues throughout the project. The current model has survived conventional controls, editor layouts, popups, scrolling, modal state, clipping, dense tables, tooltips, tiny viewports, freeform canvas objects, and transition fixtures without requiring a schema reset.

## M1 — geometry and relation derivation

**Useful first slice established; remaining work is driven by real capture evidence.**

Implemented:

- positive-area `overlaps` with intersection dimensions, area, and per-node fractions;
- `left_of` and `above` for axis-separated nodes with projection overlap;
- left/right/top/bottom edge alignment with configurable tolerance;
- deterministic symmetric endpoint ordering;
- conservative sibling-only derivation by default;
- optional cross-parent and hidden-node participation;
- stronger finite/non-negative geometry validation.

Intentional decisions:

- do not emit redundant inverse relation pairs;
- do not derive `occludes` from overlap alone;
- do not pretend bare semantic rectangles are enough for robust clipping;
- do not derive every possible pair by default because relation clouds become hostile to human and agent consumption.

The egui paint probe now provides a second geometry source: renderer-facing shape bounds plus observed clip rectangles. It can derive bounding-box clip survival, including partial and fully clipped paint submissions. This evidence remains egui-specific research data rather than canonical node geometry until a trustworthy semantic/widget association exists.

Remaining candidates:

- viewport intersection and node-level visible fraction when backed by honest coordinate/clip evidence;
- explicit coordinate-space and clip semantics;
- clipping relations once semantic/widget linkage is available;
- carefully named paint-order diagnostics weaker than full occlusion;
- relation pruning or analysis profiles for different consumption budgets.

Derived facts must remain epistemically marked as derived.

## M2 — egui capture + living showcase

**Core semantic capture, external observation, and first rendered-evidence probes established.**

Established semantic/live slice:

- optional `egui` feature so the canonical core remains usable without toolkit dependencies;
- `EguiCaptureContext` plus conversion from `egui::FullOutput` and egui-produced AccessKit `TreeUpdate`;
- semantic hierarchy reconstruction from AccessKit child relationships;
- role, name/text/value, bounds, visibility, enabled/disabled, focus, selection, toggle state, actions, and selected semantic-property mapping;
- observed `labels`, `describes`, and `controls` relations;
- deterministic node/relation ordering;
- capture provenance identifying egui + AccessKit;
- explicit identity provenance/stability plus optional application-authored identity evidence;
- executable identity probe demonstrating ordinary state stability and structure-sensitive auto identity;
- living native eframe showcase with controls, tables, scrolling, overlays, modal state, pressure switches, and custom-painted negative controls;
- optional `observer` feature with external `InspectionObserver` speaking the versioned `egui_inspection` protocol directly;
- real loopback protocol tests for handshake, TCP framing, MessagePack tree capture, scale, identity, and settle sequencing;
- bounded `settle` and `settle_and_capture`, preserving `settled: false` as evidence instead of throwing away a busy state;
- CI over both the default feature set and `--all-features`;
- an architectural rule that expensive serialization/analysis/diffing/live serving must not burden the GUI thread.

Established rendered-evidence research slice:

- provisional `EguiPaintObservation` over public `FullOutput::shapes`;
- observed flattened back-to-front renderer order;
- observed visual bounding rectangles and finite clip/scissor rectangles;
- deterministic `visible_bounds` and bounding-box `visible_fraction`;
- executable proof that a paint submission may remain in the renderer list while being partially or fully clipped;
- explicit refusal to invent AccessKit-node ↔ paint-shape identity when egui does not expose that mapping.

`docs/rendered-evidence.md` records the evidence-layer model and promotion rules. Paint observations intentionally remain outside canonical `Witness` for now.

### Living showcase

The showcase is a pressure laboratory rather than a pretty demo. Active pressure areas include:

- enabled/disabled/read-only/busy states;
- focus and keyboard navigation;
- nested/resizable panes;
- scroll areas and partially visible content;
- tooltips, menus and transient popups;
- modal dialogs and foreground/background layering;
- clipping and deliberate overflow;
- central canvas/editor layouts;
- trees, tables, lists and selection;
- long/localized text pressure;
- viewport resizing and scale changes;
- intentionally broken layouts;
- custom-painted content that AccessKit cannot describe by itself.

Where possible, each significant state should acquire an expected witness fixture, invariant, or explicit negative-control expectation.

### Live observer

The preferred production/live architecture is executable: an external ViewWitness observer consumes eframe inspection state rather than performing heavy witness work inside `App::update` or rendering paths.

Current read-only path:

```text
running eframe app
    -> egui_inspection GetTree / Settle
    -> external InspectionObserver
    -> canonical Witness
    -> derived relations / compact text / diffs
```

The next useful read-only evidence channel is screenshot retrieval as supporting raster evidence. After that, richer live paint/widget metadata will require either a lightweight ViewWitness-specific in-process reporting hook or an upstream inspection extension; the current inspection protocol does not export `FullOutput::shapes` or the complete internal `WidgetRects` table.

Observation and control remain conceptually separate even though the upstream protocol supports input injection. A witness workflow must remain usable without granting mutation authority.

## M3 — witness diff

**First useful slice established.**

`WitnessDiff` distinguishes:

- nodes added/removed;
- node fields changed;
- bounds and ordinary state changes through field-level deltas;
- relations added/removed;
- viewport changes;
- format-version changes;
- frame numbers as context without treating frame-number churn as material GUI change.

It has deterministic Rust representation plus YAML and compact agent-text projections. Executable probes cover inspector resize, transient context-menu opening, and a busy-state transition.

The identity problem is explicit rather than hidden: v0 matches by `Node.id`, while captured identity evidence records whether that ID is structure-sensitive and whether an application-authored identifier is available. Future stronger matching must expose its matching basis rather than pretending heuristic reconciliation is observation.

## M4 — operator tooling

**First useful slice established.**

The unified `viewwitness` CLI now orchestrates existing library capabilities:

```text
viewwitness validate <file>
viewwitness inspect <file> [--agent|--yaml]
viewwitness derive <file> [--agent|--yaml]
viewwitness diff <before> <after> [--agent|--yaml]
viewwitness capture [address] [--agent|--yaml] [--derive] [--settle=N]
```

Agent text is the default for repeated inspect/derive/diff/capture loops; YAML remains the richer interchange/debug projection. Subprocess tests cover the file-oriented commands, and the live-capture stack is covered by protocol integration tests.

The CLI must remain orchestration around the library, not a second model or execution layer.

## M5 — agent bridge

Expose witness capture, inspection, action targeting, and diffs to software agents. MCP remains an obvious integration surface, especially because the egui ecosystem already has inspection/MCP work, but ViewWitness must keep its canonical model independent of MCP.

Likely progression:

```text
capture -> compact query/projection -> diagnose -> optional action -> settle -> recapture -> diff
```

Do not jump directly to autonomous control before the observation/verification language is strong enough. The point of ViewWitness is to give agents better testimony about GUI reality, not merely another way to click coordinates blindly.

## Future, deliberately not scheduled

- non-egui GUI backends;
- Windows UI Automation ingestion;
- web/DOM ingestion;
- GTK/Qt adapters;
- non-Rust language bindings;
- universal GUI interchange standardization.

The architecture may permit these. The roadmap does not currently pursue them.

## When to use Codex

Architecture, ontology, format design, example design, review, and integration remain director work.

Good bounded delegation surfaces now include:

- repetitive egui/AccessKit mappings after representative cases are established;
- showcase gallery expansion against explicit acceptance cases;
- CLI polish after command behavior is fixed;
- repetitive projection formatting after the projection contract is designed;
- protocol/control plumbing once observation-versus-control semantics are settled.

The director should continue to implement small semantic slices directly when doing so helps establish the contract. Codex multiplies mechanical throughput; it does not inherit architectural authority.
