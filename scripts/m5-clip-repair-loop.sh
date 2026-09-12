#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/m5-clip-repair}"
mkdir -p "$OUT_DIR"

export DISPLAY="${DISPLAY:-:99}"
export WINIT_UNIX_BACKEND=x11
export LIBGL_ALWAYS_SOFTWARE=1

XVFB_PID=""
APP_PID=""
SOURCE_PATH="examples/showcase.rs"
HEALTHY_BINDING=$'&painter,\n                    "center",'
CLIPPED_BINDING=$'&painter.with_clip_rect(egui::Rect::from_min_max(rect.min, rect.min)),\n                    "center",'

cleanup() {
  if [[ -n "$APP_PID" ]] && kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  git checkout -- "$SOURCE_PATH" 2>/dev/null || true
  if [[ -n "$XVFB_PID" ]] && kill -0 "$XVFB_PID" 2>/dev/null; then
    kill "$XVFB_PID" 2>/dev/null || true
    wait "$XVFB_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

replace_once() {
  local old="$1"
  local new="$2"
  python3 - "$SOURCE_PATH" "$old" "$new" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
old = sys.argv[2]
new = sys.argv[3]
text = path.read_text()
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected exactly one source replacement target, found {count}: {old!r}")
path.write_text(text.replace(old, new))
PY
}

capture_state() {
  local label="$1"
  local captured=0

  VIEWWITNESS_SHOWCASE_SCENARIO=misplaced-handle \
    target/debug/examples/showcase >"$OUT_DIR/${label}-showcase.log" 2>&1 &
  APP_PID=$!

  for _attempt in $(seq 1 60); do
    if ! kill -0 "$APP_PID" 2>/dev/null; then
      echo "showcase exited before ${label} exact capture became available" >&2
      cat "$OUT_DIR/${label}-showcase.log" >&2 || true
      exit 1
    fi

    if target/debug/viewwitness capture-exact --yaml \
        >"$OUT_DIR/${label}.yaml" 2>"$OUT_DIR/${label}-capture.err"; then
      captured=1
      break
    fi
    sleep 0.5
  done

  if [[ "$captured" -ne 1 ]]; then
    echo "timed out waiting for ${label} live exact capture" >&2
    cat "$OUT_DIR/${label}-capture.err" >&2 || true
    cat "$OUT_DIR/${label}-showcase.log" >&2 || true
    exit 1
  fi

  target/debug/viewwitness inspect-exact "$OUT_DIR/${label}.yaml" \
    --object=showcase:painted-circle --binding=center \
    | tee "$OUT_DIR/${label}-center-focus.txt"

  target/debug/viewwitness inspect-exact "$OUT_DIR/${label}.yaml" \
    --object=showcase:painted-circle --binding=ring \
    | tee "$OUT_DIR/${label}-ring-focus.txt"

  grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/${label}-center-focus.txt"
  grep -q 'authored_binding_id="center"' "$OUT_DIR/${label}-center-focus.txt"
  grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/${label}-ring-focus.txt"
  grep -q 'authored_binding_id="ring"' "$OUT_DIR/${label}-ring-focus.txt"

  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  APP_PID=""
}

Xvfb "$DISPLAY" -screen 0 1280x900x24 -nolisten tcp >"$OUT_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1

# 1. Introduce a qualitatively different source defect: correct center geometry,
#    but a zero-area clip that makes only the keyed center fully invisible.
replace_once "$HEALTHY_BINDING" "$CLIPPED_BINDING"
git diff -- "$SOURCE_PATH" | tee "$OUT_DIR/source-defect.patch"
grep -q 'with_clip_rect(egui::Rect::from_min_max(rect.min, rect.min))' "$OUT_DIR/source-defect.patch"

cargo build --example showcase --features showcase
capture_state broken

# 2. Repair only the clipping source and rebuild.
replace_once "$CLIPPED_BINDING" "$HEALTHY_BINDING"
git diff -- "$SOURCE_PATH" >"$OUT_DIR/source-after-repair.diff"
if [[ -s "$OUT_DIR/source-after-repair.diff" ]]; then
  echo "clip repair did not restore the checked-in source" >&2
  cat "$OUT_DIR/source-after-repair.diff" >&2
  exit 1
fi

cargo build --example showcase --features showcase
capture_state fixed

# 3. Require ViewWitness to identify clipping, not movement, as the material repair.
target/debug/viewwitness diff-exact "$OUT_DIR/broken.yaml" "$OUT_DIR/fixed.yaml" \
  | tee "$OUT_DIR/broken-to-fixed.diff.txt"

grep -q 'authored-binding-change object_id="showcase:painted-circle" authored_binding_id="center" field="clip_rect"' \
  "$OUT_DIR/broken-to-fixed.diff.txt"

if grep -q 'authored-binding-change object_id="showcase:painted-circle" authored_binding_id="center" field="bounds"' \
    "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "center moved materially during a clipping-only repair" >&2
  exit 1
fi

if grep -q 'authored-binding-change object_id="showcase:painted-circle" authored_binding_id="ring"' \
    "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "ring was reported as a material change during center clip repair" >&2
  exit 1
fi

if grep -q 'authored-ambiguity id="showcase:painted-circle"' "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "circle authored identity became ambiguous" >&2
  exit 1
fi

if grep -q 'authored-binding-ambiguity object_id="showcase:painted-circle" authored_binding_id="center"' \
    "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "center authored identity became ambiguous" >&2
  exit 1
fi

# 4. Independently verify visibility semantics and invariant geometry from focused evidence.
python3 - "$OUT_DIR/broken-center-focus.txt" "$OUT_DIR/fixed-center-focus.txt" \
  "$OUT_DIR/broken-ring-focus.txt" "$OUT_DIR/fixed-ring-focus.txt" <<'PY'
import re
import sys
from pathlib import Path

BOUND = re.compile(r'bounds=\[([^,]+),([^,]+),([^,]+),([^\]]+)\]')
CLIP = re.compile(r'clip=\[([^,]+),([^,]+),([^,]+),([^\]]+)\]')
VISIBLE = re.compile(r'visible_fraction=([^ ]+)')

def evidence(path):
    text = Path(path).read_text()
    lines = [line for line in text.splitlines() if line.startswith('authored-binding ')]
    if len(lines) != 1:
        raise SystemExit(f"expected one authored-binding line in {path}, got {len(lines)}")
    line = lines[0]
    bound_match = BOUND.search(line)
    clip_match = CLIP.search(line)
    visible_match = VISIBLE.search(line)
    if not bound_match or not clip_match or not visible_match:
        raise SystemExit(f"missing bounds/clip/visible_fraction in {path}: {line}")
    bounds = tuple(float(value) for value in bound_match.groups())
    clip = tuple(float(value) for value in clip_match.groups())
    visible = float(visible_match.group(1))
    return bounds, clip, visible

broken_center = evidence(sys.argv[1])
fixed_center = evidence(sys.argv[2])
broken_ring = evidence(sys.argv[3])
fixed_ring = evidence(sys.argv[4])

if broken_center[0] != fixed_center[0]:
    raise SystemExit(f"center bounds changed during clip repair: {broken_center[0]} -> {fixed_center[0]}")
if broken_center[2] != 0.0:
    raise SystemExit(f"expected broken center visible_fraction=0, got {broken_center[2]}")
if fixed_center[2] != 1.0:
    raise SystemExit(f"expected repaired center visible_fraction=1, got {fixed_center[2]}")
if broken_center[1] == fixed_center[1]:
    raise SystemExit(f"center clip did not change: {broken_center[1]}")
if broken_ring != fixed_ring:
    raise SystemExit(f"ring evidence changed during center-only clip repair: {broken_ring} -> {fixed_ring}")

print(
    "M5 observed clip repair: center bounds unchanged, visible_fraction 0 -> 1, "
    f"clip {broken_center[1]} -> {fixed_center[1]}; ring unchanged"
)
PY

echo "M5 clipping source repair loop succeeded"
