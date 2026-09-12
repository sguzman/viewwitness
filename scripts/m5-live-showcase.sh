#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/m5-live-showcase}"
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
    echo "showcase exited before exact capture became available" >&2
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
  echo "timed out waiting for live exact capture" >&2
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
grep -q 'object_match_count=1 binding_match_count=1' "$OUT_DIR/outline-focus.txt"
grep -q 'authored_binding_id="outline"' "$OUT_DIR/outline-focus.txt"

echo "M5 live showcase capture succeeded: $OUT_DIR/broken.yaml"
