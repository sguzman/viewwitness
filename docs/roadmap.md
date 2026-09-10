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

**In progress.**

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

Remaining candidates before M1 is considered mature:

- viewport intersection and visible fraction;
- explicit clip rectangles / coordinate-space model;
- clipping derivation once the above exists;
- a carefully named geometric containment relation if corpus pressure justifies it;
- relation pruning or analysis profiles for different consumption budgets.

Derived facts must remain epistemically marked as derived.

## M2 — egui capture + living showcase

Build the first real backend and an egui showcase executable designed specifically to pressure ViewWitness.

The showcase should expose many controllable states rather than look pretty. Candidate galleries:

- buttons, links, toggles, radio groups, sliders and text fields;
- enabled/disabled/read-only/busy states;
- focus and keyboard navigation;
- nested panels and resizable panes;
- scroll areas and virtualized/partially visible content;
- tooltips, menus, combo boxes and transient popups;
- modal dialogs and foreground/background layering;
- clipping and deliberate overflow;
- central canvas/editor layouts;
- trees, tables, lists and selection;
- long/short/localized text pressure;
- window resizing and high-DPI scale changes;
- intentionally broken layouts.

Where possible, each showcase state should have a corresponding expected witness fixture or invariant. This turns the showcase into both a visual laboratory and a test generator.

Implementation should first exploit egui/eframe and AccessKit information that already exists rather than forking or replacing their semantics unnecessarily.

M2 is the first point where Codex is likely to be worth introducing for bounded implementation work, especially once the capture contract and showcase acceptance cases are frozen enough to delegate mechanically.

## M3 — witness diff

Introduce a first-class `WitnessDiff` over stable node identity.

It should distinguish at least:

- nodes added/removed;
- fields changed;
- bounds changed;
- relations added/removed;
- focus/selection/value transitions;
- viewport changes.

The output should have a compact agent-oriented textual projection so normal edit/verify loops do not require resending two complete snapshots.

Current executable probes:

- inspector resize: stable identity, changed geometry;
- context-menu open: stable background plus added transient nodes and semantic relation.

## M4 — operator tooling

Add the smallest useful command-line surface around capture and fixtures, tentatively:

```text
viewwitness validate <file>
viewwitness inspect <file>
viewwitness derive <file>
viewwitness diff <before> <after>
```

A live egui capture command belongs here once the backend API is stable enough.

## M5 — agent bridge

Expose witness capture, inspection, action targeting, and diffs to software agents. MCP is an obvious integration surface, especially because the egui ecosystem already has inspection/MCP work, but ViewWitness should keep its canonical model independent of MCP.

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

Codex becomes useful when a task has a bounded implementation contract and enough mechanical surface to benefit from a dedicated worker. Likely early examples are:

- implementing the egui capture adapter against a settled `Witness` contract;
- building large sections of the showcase gallery;
- filling out repetitive capture mappings once representative cases are established;
- implementing a CLI once command behavior is defined.

Geometry derivation has remained small enough to implement directly while its semantics are still being established. Do not introduce Codex merely because Rust code exists. Introduce it when there is enough grunt work to delegate without transferring architectural authority.
