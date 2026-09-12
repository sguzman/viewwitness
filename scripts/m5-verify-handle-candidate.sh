#!/usr/bin/env bash
set -euo pipefail

TRUSTED_ROOT="$(pwd -P)"
BASELINE_DIR="${1:-target/m5-separated/baseline}"
OUT_DIR="${2:-target/m5-separated/verifier}"
CANDIDATE_ROOT="${3:-.}"

mkdir -p "$OUT_DIR"
BASELINE_DIR="$(realpath "$BASELINE_DIR")"
OUT_DIR="$(realpath "$OUT_DIR")"
CANDIDATE_ROOT="$(realpath "$CANDIDATE_ROOT")"

BASELINE_YAML="$BASELINE_DIR/broken.yaml"
BASELINE_HANDLE="$BASELINE_DIR/handle-focus.txt"
BASELINE_OUTLINE="$BASELINE_DIR/outline-focus.txt"

for required in "$BASELINE_YAML" "$BASELINE_HANDLE" "$BASELINE_OUTLINE"; do
  if [[ ! -s "$required" ]]; then
    echo "missing baseline evidence: $required" >&2
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

# Build the observer CLI from the trusted checkout, then build only the candidate
# application's showcase from the candidate tree. The verifier has no source-edit
# or source-restoration authority.
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

VIEWWITNESS_SHOWCASE_SCENARIO=misplaced-handle \
  "$CANDIDATE_SHOWCASE" >"$OUT_DIR/showcase.log" 2>&1 &
APP_PID=$!

captured=0
for _attempt in $(seq 1 60); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "candidate showcase exited before exact capture became available" >&2
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
  echo "timed out waiting for candidate exact capture" >&2
  cat "$OUT_DIR/capture.err" >&2 || true
  cat "$OUT_DIR/showcase.log" >&2 || true
  exit 1
fi

"$TRUSTED_VIEWWITNESS" inspect-exact "$OUT_DIR/candidate.yaml" \
  --object=showcase:painted-rectangle --binding=handle \
  | tee "$OUT_DIR/handle-focus.txt"

"$TRUSTED_VIEWWITNESS" inspect-exact "$OUT_DIR/candidate.yaml" \
  --object=showcase:painted-rectangle --binding=outline \
  | tee "$OUT_DIR/outline-focus.txt"

for focus in "$OUT_DIR/handle-focus.txt" "$OUT_DIR/outline-focus.txt"; do
  grep -q 'object_match_count=1 binding_match_count=1' "$focus"
  grep -q 'correlation=same_full_output' "$focus"
done

grep -q 'authored_binding_id="handle"' "$OUT_DIR/handle-focus.txt"
grep -q 'authored_binding_id="outline"' "$OUT_DIR/outline-focus.txt"

"$TRUSTED_VIEWWITNESS" diff-exact "$BASELINE_YAML" "$OUT_DIR/candidate.yaml" \
  | tee "$OUT_DIR/baseline-to-candidate.diff.txt"

# The repair must be material and isolated to the named handle's geometry.
grep -q 'authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds"' \
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

# The cooperative coding agent may refactor application source, but this acceptance
# case must not perturb canonical semantic evidence either.
if grep -Eq '^(\+node |-node |change node=|\+relation |-relation )' \
    "$OUT_DIR/baseline-to-candidate.diff.txt"; then
  echo "candidate introduced collateral canonical semantic change" >&2
  cat "$OUT_DIR/baseline-to-candidate.diff.txt" >&2
  exit 1
fi

python3 - "$BASELINE_HANDLE" "$BASELINE_OUTLINE" \
  "$OUT_DIR/handle-focus.txt" "$OUT_DIR/outline-focus.txt" <<'PY'
import re
import sys
from pathlib import Path

BOUND = re.compile(r'bounds=\[([^,]+),([^,]+),([^,]+),([^\]]+)\]')


def bounds(path):
    text = Path(path).read_text()
    lines = [line for line in text.splitlines() if line.startswith('authored-binding ')]
    if len(lines) != 1:
        raise SystemExit(f"expected one authored-binding line in {path}, got {len(lines)}")
    match = BOUND.search(lines[0])
    if not match:
        raise SystemExit(f"missing bounds in {path}")
    return tuple(float(value) for value in match.groups())


baseline_handle = bounds(sys.argv[1])
baseline_outline = bounds(sys.argv[2])
candidate_handle = bounds(sys.argv[3])
candidate_outline = bounds(sys.argv[4])

if candidate_outline != baseline_outline:
    raise SystemExit(
        f"candidate moved rectangle outline: {baseline_outline} -> {candidate_outline}"
    )

if candidate_handle == baseline_handle:
    raise SystemExit("candidate handle did not materially change from broken baseline")

# Preserve handle size and vertical placement while allowing any source-level repair shape.
if candidate_handle[1:] != baseline_handle[1:]:
    raise SystemExit(
        "candidate changed handle y/size rather than repairing only horizontal attachment: "
        f"{baseline_handle} -> {candidate_handle}"
    )

handle_left, handle_y, handle_w, handle_h = candidate_handle
outline_left, outline_y, outline_w, outline_h = candidate_outline
handle_right = handle_left + handle_w
outline_right = outline_left + outline_w
horizontal_overlap = min(handle_right, outline_right) - max(handle_left, outline_left)
handle_center_y = handle_y + handle_h / 2.0
outline_center_y = outline_y + outline_h / 2.0

if horizontal_overlap <= 0:
    raise SystemExit(
        "candidate handle remains horizontally disconnected from outline: "
        f"handle={candidate_handle} outline={candidate_outline} gap={-horizontal_overlap}"
    )
if abs(handle_center_y - outline_center_y) > 0.01:
    raise SystemExit(
        "candidate handle is not vertically aligned with outline: "
        f"handle={candidate_handle} outline={candidate_outline}"
    )

print(
    "M5 independent verifier accepted candidate: handle reconnected, outline unchanged; "
    f"baseline_handle={baseline_handle} candidate_handle={candidate_handle}"
)
PY

echo "M5 mutation-free candidate verification succeeded"
