#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/m5-clip-separated/baseline}"
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

# The preceding clipping source-repair control leaves a repaired showcase binary
# behind even though its cleanup restores source. Rebuild committed source so this
# baseline is testimony about the checked-in clipped-center scenario itself.
cargo build --example showcase --features showcase

Xvfb "$DISPLAY" -screen 0 1280x900x24 -nolisten tcp >"$OUT_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1

VIEWWITNESS_SHOWCASE_SCENARIO=clipped-center \
  target/debug/examples/showcase >"$OUT_DIR/showcase.log" 2>&1 &
APP_PID=$!

captured=0
for _attempt in $(seq 1 60); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "showcase exited before clipped-center exact capture became available" >&2
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
  echo "timed out waiting for clipped-center live exact capture" >&2
  cat "$OUT_DIR/capture.err" >&2 || true
  cat "$OUT_DIR/showcase.log" >&2 || true
  exit 1
fi

target/debug/viewwitness inspect-exact "$OUT_DIR/broken.yaml" \
  --object=showcase:painted-circle --binding=center \
  | tee "$OUT_DIR/center-focus.txt"

target/debug/viewwitness inspect-exact "$OUT_DIR/broken.yaml" \
  --object=showcase:painted-circle --binding=ring \
  | tee "$OUT_DIR/ring-focus.txt"

for focus in "$OUT_DIR/center-focus.txt" "$OUT_DIR/ring-focus.txt"; do
  grep -q 'object_match_count=1 binding_match_count=1' "$focus"
  grep -q 'correlation=same_full_output' "$focus"
done

grep -q 'authored_binding_id="center"' "$OUT_DIR/center-focus.txt"
grep -q 'authored_binding_id="ring"' "$OUT_DIR/ring-focus.txt"

python3 - "$OUT_DIR/center-focus.txt" "$OUT_DIR/ring-focus.txt" <<'PY'
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


center = evidence(sys.argv[1])
ring = evidence(sys.argv[2])

if center[2] != 0.0:
    raise SystemExit(f"baseline is not broken: center visible_fraction={center[2]}")
if center[3] != "circle":
    raise SystemExit(f"baseline defect changed class: center kind={center[3]!r}")
if ring[2] != 1.0:
    raise SystemExit(f"baseline sibling ring is not fully visible: {ring[2]}")
if ring[3] != "circle":
    raise SystemExit(f"baseline sibling ring kind changed: {ring[3]!r}")

print(
    "M5 clipped-center baseline confirmed: center geometry exists but is fully clipped; "
    f"center_bounds={center[0]} center_clip={center[1]} ring_bounds={ring[0]}"
)
PY

echo "M5 mutation-free clipped-center baseline capture succeeded: $OUT_DIR/broken.yaml"
