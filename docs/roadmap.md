# ViewWitness roadmap

ViewWitness is a medium-sized infrastructure project: a durable semantic/data contract plus capture, derivation, serialization, diffing, examples, and agent-facing tooling. It is intentionally smaller in ambition than a renderer or general GUI framework.

The roadmap is evidence-driven. Concepts move forward when executable examples prove they are useful and nameable without hiding uncertainty.

## M0 — executable model exploration

**Established and permanent.**

- canonical Rust witness structs;
- YAML projection;
- structural validation;
- mini-ontology notes;
- synthetic snapshot corpus;
- transition fixtures;
- CI that parses/validates the corpus.

The corpus remains a pressure vessel for every later milestone.

## M1 — geometry and relation derivation

**Useful first slice established.**

Implemented:

- positive-area `overlaps` with intersection dimensions/area/fractions;
- `left_of` and `above` for axis-separated nodes with projection overlap;
- edge alignments with configurable tolerance;
- deterministic endpoint ordering;
- conservative sibling-only derivation by default;
- optional cross-parent/hidden participation;
- finite/non-negative geometry validation.

Intentional limits:

- no redundant inverse relation pairs;
- no canonical `occludes` from rectangle overlap;
- no fake clipping from bare semantic rectangles;
- no all-pairs relation explosion by default.

Renderer-facing egui bounds/clip evidence now forms a second, toolkit-specific geometry source. Bounding-box clip survival is deterministic derived evidence but remains outside canonical node geometry until semantic/widget linkage can justify promotion.

Remaining geometry pressure:

- semantic coordinate/clip-space modeling where direct evidence exists;
- viewport intersection/visible fraction for canonical nodes only when honestly sourced;
- paint-order diagnostics weaker than full occlusion;
- relation pruning/analysis profiles for different agent token budgets.

## M2 — egui capture + living showcase

**Core capture stack established.**

Established semantic slice:

- optional `egui` feature;
- egui/AccessKit → canonical `Witness` conversion;
- hierarchy, roles, labels/values, bounds, state, actions, and selected semantic relations;
- deterministic ordering;
- explicit identity provenance/stability;
- real probes showing generated identity can be structure-sensitive;
- native eframe showcase;
- optional external `InspectionObserver` over `egui_inspection`;
- separate screenshot evidence;
- hard GUI-thread boundary for heavy work.

Established continuous rendered-evidence slice:

- `EguiPaintObservation` over `FullOutput::shapes`;
- compact paint kind, bounds, finite clip, flattened order;
- derived bbox clip survival;
- bounded nonblocking `EguiPaintReporter`;
- executable backpressure/drop evidence;
- read-only `:5720` worker transport and external observer.

Established exact same-pass slice:

- `EguiFrameProbe` with request eligibility fixed at pass start;
- direct egui viewport evidence rather than AccessKit-root reconstruction;
- exact semantic + viewport + generic paint capture from one requested pass;
- explicit authored custom-paint object grouping;
- one or many end-of-pass-verified layer-local bindings per logical object;
- multi-layer authored bindings;
- reset/noop versus missing-handle distinction;
- optional authored binding IDs for stable rendered sub-part continuity;
- protocol v2 external exact capture on `:5721`;
- deterministic structured and agent-text projections.

Current live-showcase pressure:

- expose the already-proven authored binding IDs directly on the four Canvas sub-parts;
- use the showcase as the target of a real agent diagnose → edit → recapture → verify loop;
- keep anonymous generic paint mixed with authored evidence so instrumentation never pretends to semanticize the full renderer.

## M3 — diff and continuity

**Canonical and exact-egui slices established.**

Canonical `WitnessDiff` covers:

- nodes added/removed;
- field/bounds/state changes;
- relations added/removed;
- viewport/version changes;
- frame numbers as context rather than material change.

`EguiCorrelatedDiff` adds authored custom-paint continuity without polluting the canonical model.

Proven object rules:

- authored object ID only auto-matches when unique on both sides;
- duplicate object IDs become explicit ambiguity.

Proven binding rules:

- unique authored binding ID auto-matches within a uniquely matched object, independent of vector/submission order;
- duplicate binding IDs become explicit ambiguity;
- unkeyed bindings retain conservative relative-unkeyed-ordinal matching;
- keyed and unkeyed bindings can coexist;
- material binding state excludes `ShapeIdx`;
- pure `ShapeIdx` churn is diagnostic `execution_handle_churn`, not a logical object change.

A real egui experiment proved why this matters: inserting unrelated paint can shift `ShapeIdx` while the authored object remains unchanged. Another experiment reversed keyed sub-binding submission order and preserved `outline`/`handle` continuity while renderer slots changed.

Remaining diff pressure:

- richer real-world exact transitions from the showcase;
- avoid generic anonymous paint diffing until a trustworthy continuity source exists;
- expose matching basis whenever future reconciliation becomes more sophisticated.

## M4 — operator tooling

**Useful capture/verification loop established.**

The unified CLI includes:

```text
viewwitness validate <file>
viewwitness inspect <file> [--agent|--yaml]
viewwitness derive <file> [--agent|--yaml]
viewwitness diff <before> <after> [--agent|--yaml]
viewwitness capture [address] [--agent|--yaml] [--derive] [--settle=N]
viewwitness capture-exact [address] [--agent|--yaml] [--derive]
viewwitness diff-exact <before> <after> [--agent|--yaml]
viewwitness screenshot <output.png> [--address=HOST:PORT] [--scale=N]
```

The exact verification workflow is deliberately composable:

```text
capture-exact --yaml > before.yaml
# change/interact
capture-exact --yaml > after.yaml
diff-exact before.yaml after.yaml
```

`diff-exact` compares saved correlated captures. It does not secretly perform actions or recapture state. Agent text remains the default; YAML remains the structured interchange/debug form.

The CLI stays orchestration around library behavior, never a second model.

## M5 — agent verification loop

**Next major milestone.**

The observation vocabulary is now strong enough to pressure a real loop:

```text
capture
    -> diagnose concrete defect
    -> modify application code or perform explicit action
    -> recapture
    -> diff
    -> verify the intended state changed without hiding unrelated churn
```

The first target should be the native showcase because it already exposes conventional semantics, custom paint, exact capture, anonymous paint, clipping/layer pressure, and both semantic and rendered evidence.

MCP remains a plausible integration surface later, but the canonical model must remain independent of MCP. Observation and control must stay conceptually separate; the witness system must remain useful without mutation authority.

## Open rendered-evidence research

These are deliberately unresolved rather than papered over:

- safe mapping, if possible, from arbitrary layer-local `(LayerId, ShapeIdx)` bindings to flattened `FullOutput` global order across layer/window cases;
- stronger same-frame raster correlation;
- more multi-window/viewport exact-capture pressure;
- generic widget ↔ paint association without invented identity;
- visual occlusion stronger than rectangle overlap/paint order;
- whether repeated cross-backend pressure justifies promoting a generic authored-visual-object concept into canonical `Witness`.

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

Good bounded delegation surfaces include repetitive mappings, showcase gallery expansion after acceptance cases are fixed, CLI polish after behavior is established, and mechanical protocol/tooling work.

The director should continue implementing small semantic slices directly when that helps establish the contract. Codex multiplies mechanical throughput; it does not inherit architectural authority.
