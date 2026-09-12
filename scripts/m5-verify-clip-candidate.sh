#!/usr/bin/env bash
set -euo pipefail

TRUSTED_ROOT="$(pwd -P)"
BASELINE_DIR="${1:-target/m5-clip-separated/baseline}"
OUT_DIR="${2:-target/m5-clip-separated/verifier}"
CANDIDATE_ROOT="${3:-.}"

mkdir -p "$OUT_DIR"
BASELINE_DIR="$(realpath "$BASELINE_DIR")"
OUT_DIR="$(realpath "$OUT_DIR")"
CANDIDATE_ROOT="$(realpath "$CANDIDATE_ROOT")"

BASELINE_YAML="$BASELINE_DIR/broken.yaml"
BASELINE_CENTER="$BASELINE_DIR/center-focus.txt"
BASELINE_RING="$BASELINE_DIR/ring-focus.txt"

for required in "$BASELINE_YAML" "$BASELINE_CENTER" "$BASELINE_RING"; do
  if [[ ! -s "$required" ]]; then
    echo "missing clipped-center baseline evidence: $required" >&2
    exit 1
  fi
done

export DISPLAY="${DISPLAY:-:99}"
export WINIT_UNIX_BACKEND=x11
export LIBGL_ALWAYS_SOFTWARE=1

XVFB_PID=""
APP_PID=""

cleanup() {
  if [[ -n "$APP_PID" ]] && kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  if [[ -n "$XVFB_PID" ]] && kill -0 "$XVFB_PID" 2>/dev/null; then
    kill "$XVFB_PID" 2>/dev/null || true
    wait "$XVFB_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Build observation tooling only from the trusted checkout. Build only the
# application showcase from the candidate source tree.
(
  cd "$TRUSTED_ROOT"
  cargo build --features egui --bin viewwitness
)
TRUSTED_VIEWWITNESS="$TRUSTED_ROOT/target/debug/viewwitness"

(
  cd "$CANDIDATE_ROOT"
  cargo build --example showcase --features showcase
)
CANDIDATE_SHOWCASE="$CANDIDATE_ROOT/target/debug/examples/showcase"

Xvfb "$DISPLAY" -screen 0 1280x900x24 -nolisten tcp >"$OUT_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1

VIEWWITNESS_SHOWCASE_SCENARIO=clipped-center \
  "$CANDIDATE_SHOWCASE" >"$OUT_DIR/showcase.log" 2>&1 &
APP_PID=$!

captured=0
for _attempt in $(seq 1 60); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "candidate showcase exited before clipped-center exact capture became available" >&2
    cat "$OUT_DIR/showcase.log" >&2 || true
    exit 1
  fi

  if "$TRUSTED_VIEWWITNESS" capture-exact --yaml \
      >"$OUT_DIR/candidate.yaml" 2>"$OUT_DIR/capture.err"; then
    captured=1
    break
  fi
  sleep 0.5
done

if [[ "$captured" -ne 1 ]]; then
  echo "timed out waiting for candidate clipped-center exact capture" >&2
  cat "$OUT_DIR/capture.err" >&2 || true
  cat "$OUT_DIR/showcase.log" >&2 || true
  exit 1
fi

"$TRUSTED_VIEWWITNESS" inspect-exact "$OUT_DIR/candidate.yaml" \
  --object=showcase:painted-circle --binding=center \
  | tee "$OUT_DIR/center-focus.txt"

"$TRUSTED_VIEWWITNESS" inspect-exact "$OUT_DIR/candidate.yaml" \
  --object=showcase:painted-circle --binding=ring \
  | tee "$OUT_DIR/ring-focus.txt"

for focus in "$OUT_DIR/center-focus.txt" "$OUT_DIR/ring-focus.txt"; do
  grep -q 'object_match_count=1 binding_match_count=1' "$focus"
  grep -q 'correlation=same_full_output' "$focus"
done

grep -q 'authored_binding_id="center"' "$OUT_DIR/center-focus.txt"
grep -q 'authored_binding_id="ring"' "$OUT_DIR/ring-focus.txt"

"$TRUSTED_VIEWWITNESS" diff-exact "$BASELINE_YAML" "$OUT_DIR/candidate.yaml" \
  | tee "$OUT_DIR/baseline-to-candidate.diff.txt"

grep -q 'authored-binding-change object_id="showcase:painted-circle" authored_binding_id="center" field="clip_rect"' \
  "$OUT_DIR/baseline-to-candidate.diff.txt"

material_binding_changes=$(grep -c '^authored-binding-change ' "$OUT_DIR/baseline-to-candidate.diff.txt" || true)
if [[ "$material_binding_changes" -ne 1 ]]; then
  echo "expected exactly one material authored-binding change, found $material_binding_changes" >&2
  cat "$OUT_DIR/baseline-to-candidate.diff.txt" >&2
  exit 1
fi

if grep -Eq '^authored-(object-)?change ' "$OUT_DIR/baseline-to-candidate.diff.txt"; then
  echo "candidate introduced authored object-level material change" >&2
  cat "$OUT_DIR/baseline-to-candidate.diff.txt" >&2
  exit 1
fi

if grep -Eq '^authored-(binding-)?(add|remove) ' "$OUT_DIR/baseline-to-candidate.diff.txt"; then
  echo "candidate changed authored object/binding membership" >&2
  cat "$OUT_DIR/baseline-to-candidate.diff.txt" >&2
  exit 1
fi

if grep -Eq '^authored-(binding-)?ambiguity ' "$OUT_DIR/baseline-to-candidate.diff.txt"; then
  echo "candidate introduced authored identity ambiguity" >&2
  cat "$OUT_DIR/baseline-to-candidate.diff.txt" >&2
  exit 1
fi

python3 - "$BASELINE_CENTER" "$BASELINE_RING" \
  "$OUT_DIR/center-focus.txt" "$OUT_DIR/ring-focus.txt" <<'PY'
import re
import sys
from pathlib import Path

BOUND = re.compile(r'bounds=\[([^,]+),([^,]+),([^,]+),([^\]]+)\]')
CLIP = re.compile(r'clip=\[([^,]+),([^,]+),([^,]+),([^\]]+)\]')
VISIBLE = re.compile(r'visible_fraction=([^ ]+)')
KIND = re.compile(r'kind="([^"]+)"')


def evidence(path):
    text = Path(path).read_text()
    lines = [line for line in text.splitlines() if line.startswith('authored-binding ')]
    if len(lines) != 1:
        raise SystemExit(f"expected one authored-binding line in {path}, got {len(lines)}")
    line = lines[0]
    bound_match = BOUND.search(line)
    clip_match = CLIP.search(line)
    visible_match = VISIBLE.search(line)
    kind_match = KIND.search(line)
    if not bound_match or not clip_match or not visible_match or not kind_match:
        raise SystemExit(f"missing bounds/clip/visible_fraction/kind in {path}: {line}")
    return (
        tuple(float(value) for value in bound_match.groups()),
        tuple(float(value) for value in clip_match.groups()),
        float(visible_match.group(1)),
        kind_match.group(1),
    )


baseline_center = evidence(sys.argv[1])
baseline_ring = evidence(sys.argv[2])
candidate_center = evidence(sys.argv[3])
candidate_ring = evidence(sys.argv[4])

if baseline_center[2] != 0.0:
    raise SystemExit(f"baseline center is not fully clipped: {baseline_center}")
if candidate_center[0] != baseline_center[0]:
    raise SystemExit(
        f"candidate moved center geometry: {baseline_center[0]} -> {candidate_center[0]}"
    )
if candidate_center[3] != baseline_center[3]:
    raise SystemExit(
        f"candidate changed center kind: {baseline_center[3]!r} -> {candidate_center[3]!r}"
    )
if candidate_center[2] != 1.0:
    raise SystemExit(
        f"candidate center is not fully visible after repair: visible_fraction={candidate_center[2]}"
    )
if candidate_center[1] == baseline_center[1]:
    raise SystemExit(f"candidate did not change center clipping evidence: {candidate_center[1]}")
if candidate_ring != baseline_ring:
    raise SystemExit(
        f"candidate changed sibling ring evidence: {baseline_ring} -> {candidate_ring}"
    )

print(
    "M5 independent clip verifier accepted candidate: center became fully visible, "
    f"center_bounds={candidate_center[0]}, ring unchanged"
)
PY

echo "M5 mutation-free clipped-center candidate verification succeeded"
