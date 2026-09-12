#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-target/m5-separated/patcher}"
mkdir -p "$OUT_DIR"

BROKEN_EXPR='first.right_center() + egui::vec2(60.0, 0.0)'
FIXED_EXPR='first.right_center() + egui::vec2(0.0, 0.0)'

mapfile -t matches < <(grep -RIlF --include='*.rs' "$BROKEN_EXPR" . \
  --exclude-dir=.git --exclude-dir=target)

if [[ "${#matches[@]}" -ne 1 ]]; then
  printf 'reference patcher expected exactly one source file containing the repair target, found %d\n' \
    "${#matches[@]}" >&2
  printf '%s\n' "${matches[@]:-}" >&2
  exit 1
fi

SOURCE_PATH="${matches[0]#./}"
printf '%s\n' "$SOURCE_PATH" >"$OUT_DIR/located-source.txt"

python3 - "$SOURCE_PATH" "$BROKEN_EXPR" "$FIXED_EXPR" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
old = sys.argv[2]
new = sys.argv[3]
text = path.read_text()
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected exactly one repair target in {path}, found {count}")
path.write_text(text.replace(old, new))
PY

git diff -- "$SOURCE_PATH" | tee "$OUT_DIR/candidate.patch"

if [[ ! -s "$OUT_DIR/candidate.patch" ]]; then
  echo "reference patcher produced no candidate diff" >&2
  exit 1
fi

echo "M5 reference patcher produced candidate source tree"
