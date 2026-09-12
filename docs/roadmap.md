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

Renderer-facing egui bounds/clip evidence forms a second, toolkit-specific geometry source. Bounding-box clip survival is deterministic derived evidence but remains outside canonical node geometry until semantic/widget linkage can justify promotion.

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

The live Canvas pressure case exposes and guards four keyed bindings directly:

```text
showcase:painted-rectangle -> outline, handle
showcase:painted-circle    -> ring, center
```

Canvas background/text remain anonymous generic paint so instrumentation never pretends to semanticize the entire renderer.

The showcase also contains a guarded `Misplaced canvas handle` pressure switch. When enabled, only the rectangle's keyed `handle` is displaced while its keyed `outline` remains fixed. This is the canonical live broken-state target for M5 and is deterministically launchable with `VIEWWITNESS_SHOWCASE_SCENARIO=misplaced-handle`.

## M3 — diff and continuity

**Canonical and exact-egui slices established.**

Canonical `WitnessDiff` covers nodes, fields/state/bounds, relations, viewport/version changes, and frame context without treating frame-number churn as material state.

`EguiCorrelatedDiff` adds authored custom-paint continuity without polluting the canonical model.

Proven object rules:

- authored object ID only auto-matches when unique on both sides;
- duplicate object IDs become explicit ambiguity;
- object-level changes cover object semantics (`role`, `name`, semantic provenance), not an opaque binding collection.

Proven binding rules:

- unique authored binding ID auto-matches within a uniquely matched object, independent of vector/submission order;
- duplicate binding IDs become explicit ambiguity;
- unkeyed bindings retain conservative relative-unkeyed-ordinal matching;
- keyed and unkeyed bindings can coexist;
- binding additions/removals are first-class records;
- material binding changes are field-granular (`kind`, `bounds`, `clip_rect`, layer, verification, evidence);
- material binding state excludes `ShapeIdx`;
- pure `ShapeIdx` churn is diagnostic `execution_handle_churn`, not a logical change.

A real egui experiment proved unrelated prefix paint can shift `ShapeIdx` while authored state remains unchanged. Another reversed keyed submission order while preserving `outline`/`handle` identity.

Remaining diff pressure:

- richer exact transitions from real application/showcase defects;
- focused diff projection if future agent loops demonstrate that full correlated diffs are unnecessarily noisy;
- avoid generic anonymous paint diffing until a trustworthy continuity source exists;
- expose matching basis whenever future reconciliation becomes more sophisticated.

## M4 — operator tooling

**Useful capture/inspection/verification loop established.**

The unified CLI includes:

```text
viewwitness validate <file>
viewwitness inspect <file> [--agent|--yaml]
viewwitness inspect-exact <file> [--agent|--yaml] [--object=ID [--binding=ID]]
viewwitness derive <file> [--agent|--yaml]
viewwitness diff <before> <after> [--agent|--yaml]
viewwitness capture [address] [--agent|--yaml] [--derive] [--settle=N]
viewwitness capture-exact [address] [--agent|--yaml] [--derive] [--object=ID [--binding=ID]]
viewwitness diff-exact <before> <after> [--agent|--yaml]
viewwitness screenshot <output.png> [--address=HOST:PORT] [--scale=N]
```

The full exact workflow remains deliberately composable:

```text
capture-exact --yaml > before.yaml
# change/interact
capture-exact --yaml > after.yaml
diff-exact before.yaml after.yaml
```

Focused exact inspection is a smaller **agent-text projection**, not a replacement envelope:

```text
inspect-exact before.yaml --object=showcase:painted-rectangle
inspect-exact before.yaml --object=showcase:painted-rectangle --binding=handle
capture-exact --object=showcase:painted-rectangle --binding=handle
```

The focused header retains request/pass/viewport correlation metadata, reports object/binding match counts, and explicitly says `projection=authored_focus` plus `omitted=canonical_semantics,generic_paint`. Duplicate object or binding IDs remain multiple visible matches; zero matches remain explicit. Focused YAML is rejected because that projection is not a complete correlated envelope. `--binding` requires `--object`, and focused inspection cannot be combined with canonical geometry derivation.

`diff-exact` compares saved correlated captures only. It does not secretly perform actions or recapture state. Agent text remains the default; YAML remains the structured interchange/debug form for complete envelopes and diffs.

## M5 — agent verification loop

**First live source-edit acceptance established.**

The target workflow is:

```text
capture
    -> diagnose concrete defect
    -> modify application source
    -> rebuild/restart
    -> recapture
    -> diff
    -> verify the intended state changed without hiding unrelated churn
```

Earlier real-egui probes established the evidence language in isolation: one authored `diagram_node` with keyed `body` and `handle` parts could be repaired while unrelated prefix paint changed renderer slots. The accepted diff isolated `handle.bounds` as material and renderer-handle churn as non-material.

That evidence model has now crossed the **live native source-edit boundary**. The dedicated `M5 Live Source Repair` workflow runs the real eframe showcase under Xvfb and executes this sequence in one isolated workspace:

```text
launch checked-in broken showcase
    -> exact live capture
    -> focused inspect showcase:painted-rectangle / handle
    -> edit examples/showcase.rs
    -> rebuild native showcase
    -> relaunch same deterministic scenario
    -> exact live recapture
    -> diff-exact broken.yaml fixed.yaml
    -> machine-check the repair
```

The accepted source edit was:

```diff
-        first.right_center() + egui::vec2(60.0, 0.0)
+        first.right_center() + egui::vec2(0.0, 0.0)
```

Observed exact evidence on the accepted run:

```text
broken handle:  [513,215,10,10]
fixed handle:   [453,215,10,10]
outline before: [307,169,152,102]
outline after:  [307,169,152,102]
```

`diff-exact` reported exactly the named rendered sub-part material change:

```text
authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds" before={"height":10.0,"width":10.0,"x":513.0,"y":215.0} after={"height":10.0,"width":10.0,"x":453.0,"y":215.0}
```

The harness rejects outline material movement, object/binding ambiguity, movement other than 60 logical pixels left, or any handle change outside its x position. All assertions passed.

The checked-in showcase intentionally remains broken. The workflow repairs only its isolated checkout and restores the file during cleanup, preserving a deterministic defect for repeated experiments.

Canonical acceptance details and artifact provenance live in `docs/acceptance/m5-live-source-repair.md`. The executable orchestration lives in `scripts/m5-source-repair-loop.sh` and `.github/workflows/m5-live-showcase.yml`.

This **does not yet prove arbitrary autonomous GUI repair**. The repair target and transformation are deliberately constrained. The next M5 pressure is to remove scaffolding in stages:

- give an agent the live focused evidence and repository without pre-encoding the exact replacement string in the harness;
- require it to locate the responsible source from evidence plus code search;
- let the agent choose and apply the patch through the normal coding workflow;
- rerun the same external verification independently of the patching agent;
- add a second defect class so success cannot collapse into memorizing one handle-offset recipe;
- eventually test diagnosis where semantic, authored-paint, and raster evidence disagree or are incomplete.

MCP remains a plausible integration surface later, but the canonical model must remain independent of MCP. Observation and control stay conceptually separate; ViewWitness must remain useful without mutation authority.

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
