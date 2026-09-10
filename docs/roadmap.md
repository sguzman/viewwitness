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

**Useful first slice established; remaining work is now driven by real egui evidence.**

Teach ViewWitness to derive deterministic facts from captured geometry rather than hand-authoring them in fixtures.

Implemented first slice:

- positive-area `overlaps` with intersection dimensions, area, and per-node fractions;
- `left_of` and `above` for axis-separated nodes with projection overlap;
- left/right/top/bottom edge alignment with configurable tolerance;
- deterministic symmetric endpoint ordering;
- conservative sibling-only derivation by default;
- optional cross-parent and hidden-node participation;
- stronger finite/non-negative geometry validation.

Intentional decisions from corpus pressure:

- do not emit redundant inverse pairs (`left_of` does not require an additional `right_of` fact);
- do not derive `occludes` from overlap alone because z-order/paint evidence is required;
- do not pretend bare rectangles are enough for robust clipping while scroll and clip coordinate spaces remain underspecified;
- do not derive every possible pair by default because relation clouds become hostile to human and agent consumption.

Remaining candidates are intentionally coupled to M2 capture work:

- viewport intersection and visible fraction;
- explicit clip rectangles / coordinate-space model;
- clipping derivation once the above exists;
- a carefully named geometric containment relation if corpus pressure justifies it;
- relation pruning or analysis profiles for different consumption budgets.

Derived facts must remain epistemically marked as derived.

## M2 — egui capture + living showcase

**Core semantic and live-observation slice established; rendered-evidence expansion remains active.**

Established:

- optional `egui` crate feature so the canonical core remains usable without toolkit dependencies;
- `EguiCaptureContext` plus conversion from `egui::FullOutput` and egui-produced AccessKit `TreeUpdate`;
- semantic hierarchy reconstruction from AccessKit child relationships;
- role, name/text/value, bounds, visibility, enabled/disabled, focus, selection, toggle state, actions, and selected semantic-property mapping;
- observed `labels`, `describes`, and `controls` relations;
- deterministic node/relation ordering;
- capture provenance identifying egui + AccessKit;
- explicit identity provenance and stability rather than assuming backend IDs are permanent conceptual identities;
- an executable identity probe demonstrating ordinary state stability and structure-sensitive auto identity;
- headless tests using both constructed AccessKit trees and actual egui frame output;
- end-to-end `examples/egui_capture.rs` that emits ViewWitness YAML from a real headless egui frame;
- a living native eframe showcase with controls, tables, scrolling, overlays, modal state, pressure switches, and a custom-painted negative control;
- optional `observer` feature with external `InspectionObserver` speaking the versioned `egui_inspection` protocol directly;
- one-shot `examples/inspection_capture.rs` for a running inspected egui application;
- a loopback protocol integration test exercising the real handshake, TCP framing, MessagePack request/response, AccessKit tree conversion, scale, and author identity;
- CI over both the default feature set and `--all-features`;
- an explicit architectural rule that expensive serialization/analysis/diffing/live serving must not burden the GUI thread.

The adapter deliberately preserves rather than conceals uncertainty. AccessKit transforms are detected but not yet composed into canonical bounds, `clips_children` is observed without pretending it gives enough information for exact clipping geometry, and the external observer refuses to guess viewport geometry if usable root bounds are unavailable.

### Living showcase

The first showcase surface exists. Its continuing role is to pressure ViewWitness rather than to become a pretty example application. Active pressure areas include:

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

Where possible, each significant showcase state should acquire an expected witness fixture, invariant, or explicit negative-control expectation.

### Live observer

The preferred production/live architecture is now executable: an external ViewWitness observer consumes eframe inspection state rather than performing heavy witness work inside `App::update` or rendering paths.

Current read-only path:

```text
running eframe app
    -> egui_inspection GetTree
    -> external InspectionObserver
    -> canonical Witness
```

The observer should next grow only where concrete agent workflows require it. Candidate additions:

- settle-then-capture using inspection `Settle`;
- optional screenshot retrieval as supporting evidence;
- explicit viewport override/fallback only if native experiments prove root bounds insufficient;
- richer captured layer/clip evidence;
- orchestration that derives geometry and diffs outside the GUI process.

Observation and control should remain conceptually separable even though the upstream inspection protocol supports input injection. A read-only witness path should remain usable without granting an agent mutation authority.

The canonical model remains independent of eframe inspection, AccessKit, and MCP. They are evidence/transport integrations around `Witness`.

## M3 — witness diff

**First useful slice established.**

`WitnessDiff` now distinguishes:

- nodes added/removed;
- node fields changed;
- bounds and ordinary state changes through field-level deltas;
- relations added/removed;
- viewport changes;
- format-version changes;
- frame numbers as context without treating frame-number churn as a material GUI change.

It has deterministic Rust representation plus YAML serialization/deserialization. Current executable probes cover:

- inspector resize: stable identity, changed geometry;
- context-menu open: stable background plus added transient nodes and semantic relation;
- busy transition: an existing action becomes disabled while progress/status state appears.

The remaining identity problem is now explicit rather than hidden: v0 matches by `Node.id`, while captured identity evidence records whether that ID is structure-sensitive and whether an application-authored identifier is available. Future stronger matching must expose its matching basis rather than pretending heuristic reconciliation is observation.

The next M3 pressure is a **compact agent-oriented textual projection** so edit/verify loops do not require verbose YAML when only a small transition matters.

## M4 — operator tooling

**Now becoming the next integration surface.**

The project already has runnable examples for headless capture, live inspection capture, and witness diffing. The next step is to consolidate the stable pieces into the smallest useful command-line surface, tentatively:

```text
viewwitness validate <file>
viewwitness inspect <file>
viewwitness derive <file>
viewwitness diff <before> <after>
viewwitness capture [inspection-address]
```

The CLI should orchestrate existing library capabilities rather than become a second model or execution layer.

## M5 — agent bridge

Expose witness capture, inspection, action targeting, and diffs to software agents. MCP remains an obvious integration surface, especially because the egui ecosystem already has inspection/MCP work, but ViewWitness must keep its canonical model independent of MCP.

A likely progression is:

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

Codex is useful when a task has a bounded implementation contract and enough mechanical surface to benefit from a dedicated worker. Current good delegation surfaces include:

- filling out repetitive egui/AccessKit mappings after representative cases are established;
- expanding showcase galleries against explicit acceptance cases;
- implementing a CLI once command behavior is fixed;
- implementing repetitive compact-projection formatting after the projection contract is designed;
- filling protocol/control plumbing once observation-versus-control semantics are settled.

The director should continue to implement small semantic slices directly when doing so helps establish the contract. Codex multiplies mechanical throughput; it does not inherit architectural authority.
