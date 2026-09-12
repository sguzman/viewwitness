#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/m5-separated/baseline}"
mkdir -p "$OUT_DIR"

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

Xvfb "$DISPLAY" -screen 0 1280x900x24 -nolisten tcp >"$OUT_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1

VIEWWITNESS_SHOWCASE_SCENARIO=misplaced-handle \
  target/debug/examples/showcase >"$OUT_DIR/showcase.log" 2>&1 &
APP_PID=$!

captured=0
for _attempt in $(seq 1 60); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "showcase exited before broken exact capture became available" >&2
    cat "$OUT_DIR/showcase.log" >&2 || true
    exit 1
  fi

  if target/debug/viewwitness capture-exact --yaml \
      >"$OUT_DIR/broken.yaml" 2>"$OUT_DIR/capture.err"; then
    captured=1
    break
  fi
  sleep 0.5
done

if [[ "$captured" -ne 1 ]]; then
  echo "timed out waiting for broken live exact capture" >&2
  cat "$OUT_DIR/capture.err" >&2 || true
  cat "$OUT_DIR/showcase.log" >&2 || true
  exit 1
fi

target/debug/viewwitness inspect-exact "$OUT_DIR/broken.yaml" \
  --object=showcase:painted-rectangle --binding=handle \
  | tee "$OUT_DIR/handle-focus.txt"

target/debug/viewwitness inspect-exact "$OUT_DIR/broken.yaml" \
  --object=showcase:painted-rectangle --binding=outline \
  | tee "$OUT_DIR/outline-focus.txt"

grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/handle-focus.txt"
grep -q 'authored_binding_id="handle"' "$OUT_DIR/handle-focus.txt"
grep -q 'correlation=same_full_output' "$OUT_DIR/handle-focus.txt"
grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/outline-focus.txt"
grep -q 'authored_binding_id="outline"' "$OUT_DIR/outline-focus.txt"

python3 - "$OUT_DIR/handle-focus.txt" "$OUT_DIR/outline-focus.txt" <<'PY'
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

handle = bounds(sys.argv[1])
outline = bounds(sys.argv[2])
handle_left, handle_y, handle_w, handle_h = handle
outline_left, outline_y, outline_w, outline_h = outline
handle_right = handle_left + handle_w
outline_right = outline_left + outline_w
horizontal_overlap = min(handle_right, outline_right) - max(handle_left, outline_left)
handle_center_y = handle_y + handle_h / 2.0
outline_center_y = outline_y + outline_h / 2.0

if horizontal_overlap > 0:
    raise SystemExit(
        f"baseline is not broken: handle already overlaps outline horizontally: {handle} vs {outline}"
    )
if abs(handle_center_y - outline_center_y) > 0.01:
    raise SystemExit(
        f"baseline defect changed class: handle is no longer vertically aligned: {handle} vs {outline}"
    )

print(
    "M5 broken baseline confirmed: handle is vertically aligned but horizontally disconnected; "
    f"handle={handle} outline={outline} gap={-horizontal_overlap}"
)
PY

echo "M5 mutation-free broken baseline capture succeeded: $OUT_DIR/broken.yaml"
