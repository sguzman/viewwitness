#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/m5-source-repair}"
mkdir -p "$OUT_DIR"

export DISPLAY="${DISPLAY:-:99}"
export WINIT_UNIX_BACKEND=x11
export LIBGL_ALWAYS_SOFTWARE=1

XVFB_PID=""
APP_PID=""
SOURCE_PATH="examples/showcase.rs"
BROKEN_EXPR='first.right_center() + egui::vec2(60.0, 0.0)'
FIXED_EXPR='first.right_center() + egui::vec2(0.0, 0.0)'

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
    --object=showcase:painted-rectangle --binding=handle \
    | tee "$OUT_DIR/${label}-handle-focus.txt"

  target/debug/viewwitness inspect-exact "$OUT_DIR/${label}.yaml" \
    --object=showcase:painted-rectangle --binding=outline \
    | tee "$OUT_DIR/${label}-outline-focus.txt"

  grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/${label}-handle-focus.txt"
  grep -q 'authored_binding_id="handle"' "$OUT_DIR/${label}-handle-focus.txt"
  grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/${label}-outline-focus.txt"
  grep -q 'authored_binding_id="outline"' "$OUT_DIR/${label}-outline-focus.txt"

  kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  APP_PID=""
}

Xvfb "$DISPLAY" -screen 0 1280x900x24 -nolisten tcp >"$OUT_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1

# 1. Observe the checked-in broken application before touching source.
capture_state broken

# 2. Apply the source repair derived from the live focused evidence.
python3 - "$SOURCE_PATH" "$BROKEN_EXPR" "$FIXED_EXPR" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
old = sys.argv[2]
new = sys.argv[3]
text = path.read_text()
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected exactly one repair target, found {count}: {old!r}")
path.write_text(text.replace(old, new))
PY

git diff -- "$SOURCE_PATH" | tee "$OUT_DIR/source-repair.patch"
grep -q -- '-        first.right_center() + egui::vec2(60.0, 0.0)' "$OUT_DIR/source-repair.patch"
grep -q -- '+        first.right_center() + egui::vec2(0.0, 0.0)' "$OUT_DIR/source-repair.patch"

# 3. Rebuild the real native application from the repaired source.
cargo build --example showcase --features showcase

# 4. Observe the rebuilt application through the same exact-capture path.
capture_state fixed

# 5. Let ViewWitness itself decide what materially changed.
target/debug/viewwitness diff-exact "$OUT_DIR/broken.yaml" "$OUT_DIR/fixed.yaml" \
  | tee "$OUT_DIR/broken-to-fixed.diff.txt"

grep -q 'authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="handle" field="bounds"' \
  "$OUT_DIR/broken-to-fixed.diff.txt"

if grep -q 'authored-binding-change object_id="showcase:painted-rectangle" authored_binding_id="outline"' \
    "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "outline was reported as a material authored-binding change" >&2
  exit 1
fi

if grep -q 'authored-ambiguity id="showcase:painted-rectangle"' "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "rectangle authored identity became ambiguous" >&2
  exit 1
fi

if grep -q 'authored-binding-ambiguity object_id="showcase:painted-rectangle" authored_binding_id="handle"' \
    "$OUT_DIR/broken-to-fixed.diff.txt"; then
  echo "handle authored identity became ambiguous" >&2
  exit 1
fi

# 6. Verify direction/magnitude from the focused observed bounds, independently of source text.
python3 - "$OUT_DIR/broken-handle-focus.txt" "$OUT_DIR/fixed-handle-focus.txt" \
  "$OUT_DIR/broken-outline-focus.txt" "$OUT_DIR/fixed-outline-focus.txt" <<'PY'
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

broken_handle = bounds(sys.argv[1])
fixed_handle = bounds(sys.argv[2])
broken_outline = bounds(sys.argv[3])
fixed_outline = bounds(sys.argv[4])

movement_x = broken_handle[0] - fixed_handle[0]
if abs(movement_x - 60.0) > 0.01:
    raise SystemExit(
        f"expected repaired handle to move left by 60 px, observed {movement_x}: "
        f"{broken_handle} -> {fixed_handle}"
    )
if broken_handle[1:] != fixed_handle[1:]:
    raise SystemExit(f"handle changed outside x position: {broken_handle} -> {fixed_handle}")
if broken_outline != fixed_outline:
    raise SystemExit(f"outline bounds changed: {broken_outline} -> {fixed_outline}")

print(
    "M5 observed repair: handle moved left 60 px, outline bounds unchanged; "
    f"handle {broken_handle} -> {fixed_handle}"
)
PY

echo "M5 source repair loop succeeded"
