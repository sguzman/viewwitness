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

The showcase contains deterministic pressure scenarios for both a misplaced rectangle handle and a clipped circle center. They provide two qualitatively different rendered-defect classes for M5 rather than one memorized offset recipe.

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

**Stages A–D accepted. Stage-E agent arena dual-task validated. Real agent-produced repair pending.**

The durable target loop is:

```text
capture
    -> diagnose concrete defect
    -> coding agent chooses source repair
    -> rebuild/restart candidate
    -> recapture with trusted observer
    -> diff
    -> verify intended state changed without hiding unrelated authored churn
```

### Accepted evidence and source-repair slices

The first native repair class is a rectangle whose keyed `handle` is horizontally disconnected from its keyed `outline`. Exact rendered evidence isolates `handle.bounds` as the intended material change while the outline remains stable.

The second repair class is qualitatively different: a circle's keyed `center` is clipped to zero visible area while the keyed `ring` remains visible. Its accepted repair changes clip evidence rather than object geometry. This prevents M5 from collapsing into one memorized handle-offset recipe.

These live source-edit experiments established Stages B and C, but their patch transformations were still harness-controlled.

### Stage D — patcher/verifier separation

**Accepted.**

The handle case was split into three executable actors:

```text
mutation-free baseline observer
    -> isolated reference patcher
    -> mutation-free independent verifier
```

The verifier does not contain the expected source file, broken expression, replacement expression, patch application, or source restoration logic. It accepts a candidate from observed state predicates instead:

- rectangle, `outline`, and `handle` remain uniquely identifiable;
- exact capture remains `same_full_output`;
- outline geometry remains stable;
- handle y-position and size remain stable;
- handle materially changes from the broken baseline;
- candidate handle geometrically reconnects to the outline;
- no extra authored-binding material change or identity ambiguity is introduced.

The verifier deliberately does **not** require a particular source-level displacement magnitude or patch shape.

Canonical Stage-D provenance lives in `docs/acceptance/m5-separated-patcher-verifier.md`.

### Stage E — recipe-free coding-agent boundary

**Dual-task arena validated; Stage E itself remains open.**

Stage E removes the repair recipe from the coding-agent side as well. There are now two recipe-free task surfaces:

```text
handle
  prompt: prompts/m5-handle-repair.md
  focus:  handle + outline

clip
  prompt: prompts/m5-clip-repair.md
  focus:  center + ring
```

Both tasks describe broken rendered state and acceptance goals rather than expected source paths or transformations.

For either task, the agent-visible package is generated from committed broken `HEAD` and contains:

```text
TASK.md
task-kind.txt
evidence/broken.yaml
evidence/<task-specific focused projections>
workspace/Cargo.toml
workspace/src/**
workspace/examples/**
```

Historical tests, docs, scripts, prompts, CI, and verifier machinery are excluded so the coding agent cannot simply read previous answers out of the repository's acceptance history.

A candidate patch then crosses a trusted boundary:

```text
agent.patch + explicit task kind
    -> select trusted handle|clip verifier
    -> validate mutation surface
    -> fresh detached worktree from trusted HEAD
    -> apply patch only there
    -> build candidate showcase there
    -> trusted ViewWitness CLI observes candidate
    -> selected trusted verifier decides acceptance
```

For the current experiment the candidate mutation surface is restricted to `examples/*.rs`, keeping ViewWitness implementation, tests, CI, task text, and verifier outside agent authority.

The code-bearing dual-task integration is validated at `1a526beb26aa97e800fa32b481861d351410abde`. Native M5 run `34725470386` passed both mutation-free baselines, both sanitized-workspace builds, and both trusted clean-worktree control gates. The handle control was accepted from geometry evidence. The clip control was accepted from a `visible_fraction` transition from `0` to `1` with center bounds and sibling ring evidence stable.

The post-integration test head `4cd6bbfc64733c9077c9b0652ec49607375ba559` passed ordinary formatting, default tests, and all-features CI in run `34725563814`.

That distinction remains load-bearing: **known-good controls passing the Stage-E gate validate the arena, not Stage-E agent inference.**

Canonical arena provenance lives in `docs/acceptance/m5-stage-e-arena.md`. The trust contract lives in `docs/agent-patch-contract.md`.

A native pressure run also showed why acceptance must remain epistemically scoped: AccessKit debug-inspector nodes can reflow across process restarts because generated identity/layout is structure-sensitive. Those canonical semantic diffs remain preserved evidence, but they are not promoted into a blanket collateral-change veto for an authored custom-paint repair whose stronger continuity source is its explicit authored object/binding identity.

### Next M5 pressure

The missing step is now singular: put an **actual coding agent** inside the already-validated arena.

```text
real coding agent
    receives one sanitized Stage-E package
    -> reads ViewWitness evidence
    -> inspects ordinary source
    -> locates responsible code itself
    -> chooses and emits its own patch
    -> trusted task-specific gate reconstructs and verifies candidate
```

Only successful completion of that experiment promotes Stage E from `current` to `accepted`.

After that, pressure generality rather than adding more control scripts:

```text
Stage F  same independent-agent protocol succeeds across multiple defect classes/tasks
Stage G  incomplete, contradictory, or ambiguous evidence pressure
```

Stage F should require successful independently chosen repairs for more than one task kind so success cannot degrade into memorizing one source idiom. Stage G should deliberately pressure disagreement or incompleteness among semantic, authored-paint, geometry, and raster evidence and require uncertainty to remain visible rather than guessed away.

MCP remains a plausible integration surface later, but the canonical model and verification architecture must remain independent of MCP. Observation and control stay conceptually separate; ViewWitness must remain useful without mutation authority.

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

A real Stage-E run is now the strongest Codex delegation surface precisely because the trust boundary is external to the coding agent: Codex may own the repair hypothesis, while ViewWitness retains independent observation and acceptance authority.

The director should continue implementing small semantic slices directly when that helps establish the contract. Codex multiplies mechanical throughput; it does not inherit architectural authority.
