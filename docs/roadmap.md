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

The showcase also contains a guarded `Misplaced canvas handle` pressure switch. When enabled, only the rectangle's keyed `handle` is displaced while its keyed `outline` remains fixed. This is now the canonical live broken-state target for M5.

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
- focused diff projection if the real agent loop demonstrates that full correlated diffs are unnecessarily noisy;
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

**First executable slice plus live broken target established.**

The target workflow is:

```text
capture
    -> diagnose concrete defect
    -> modify application code or perform explicit action
    -> recapture
    -> diff
    -> verify the intended state changed without hiding unrelated churn
```

The first real-egui verification probe executes the evidence half of this loop end to end. It captures one authored `diagram_node` with keyed `body` and `handle` parts in a broken state, then captures a fixed state after also inserting unrelated prefix paint.

The accepted result is intentionally precise:

```text
material:     handle.bounds changed
non-material: body ShapeIdx churned because unrelated paint shifted renderer slots
false noise:  no body material change
false blob:   no object-level field="bindings"
```

This forced a useful contract improvement: authored binding changes are first-class and field-granular, so an agent can see exactly which named rendered sub-part changed rather than re-diffing an opaque binding collection itself.

The same defect shape now exists in the **running showcase**. On the Canvas page, `Misplaced canvas handle` displaces only `showcase:painted-rectangle`'s keyed `handle`. Focused exact inspection can isolate that object or binding without forcing an agent to consume the entire semantic tree and generic renderer list.

What M5 has **not** yet proven is autonomous source modification. The next acceptance case is now concrete rather than hypothetical:

```text
run showcase
    -> enable Misplaced canvas handle
    -> capture full broken envelope
    -> focus inspection on showcase:painted-rectangle / handle
    -> diagnose source responsible for the displacement
    -> edit source outside the GUI thread
    -> rebuild/restart as needed
    -> recapture full envelope
    -> diff-exact broken vs fixed
    -> verify the named handle changed as intended without unrelated false material changes
```

Turning the pressure switch off can prove live state/action verification, but it does **not** count as source-edit proof. The source-edit milestone remains open until the application code itself is changed and the recaptured evidence demonstrates the fix.

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
