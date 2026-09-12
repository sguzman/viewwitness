#!/usr/bin/env bash
set -euo pipefail

BASELINE_DIR="${1:-target/m5-separated/baseline}"
PATCH_FILE="${2:?usage: m5-verify-agent-patch.sh BASELINE_DIR PATCH_FILE [OUT_DIR] [TASK_KIND]}"
OUT_DIR="${3:-target/m5-stage-e/verifier}"
TASK_KIND="${4:-handle}"

TRUSTED_ROOT="$(pwd -P)"

case "$TASK_KIND" in
  handle)
    VERIFIER_NAME="m5-verify-handle-candidate.sh"
    ;;
  clip)
    VERIFIER_NAME="m5-verify-clip-candidate.sh"
    ;;
  *)
    echo "unknown Stage E task kind: $TASK_KIND (expected handle or clip)" >&2
    exit 1
    ;;
esac
VERIFIER_PATH="$TRUSTED_ROOT/scripts/$VERIFIER_NAME"

mkdir -p "$OUT_DIR"
BASELINE_DIR="$(realpath "$BASELINE_DIR")"
PATCH_FILE="$(realpath "$PATCH_FILE")"
OUT_DIR="$(realpath "$OUT_DIR")"

if [[ ! -s "$PATCH_FILE" ]]; then
  echo "candidate patch is missing or empty: $PATCH_FILE" >&2
  exit 1
fi

# Stage E protects the observation/verifier stack from the coding agent. The
# candidate may change Rust application examples, but not ViewWitness library,
# verifier, tests, CI, docs, or evidence artifacts.
python3 - "$PATCH_FILE" "$OUT_DIR/candidate-paths.txt" <<'PY'
import re
import sys
from pathlib import Path

patch = Path(sys.argv[1]).read_text()
out = Path(sys.argv[2])
paths = []
for line in patch.splitlines():
    if not line.startswith("diff --git "):
        continue
    match = re.fullmatch(r"diff --git a/(.+) b/(.+)", line)
    if not match:
        raise SystemExit(f"unsupported diff header: {line}")
    before, after = match.groups()
    if before != after:
        raise SystemExit(f"renames are outside this acceptance surface: {before} -> {after}")
    if before.startswith("/") or ".." in Path(before).parts:
        raise SystemExit(f"unsafe candidate path: {before}")
    if not before.startswith("examples/") or not before.endswith(".rs"):
        raise SystemExit(
            "candidate patch touched trusted/non-application surface: " + before
        )
    paths.append(before)

if not paths:
    raise SystemExit("candidate patch contains no changed files")

unique = sorted(set(paths))
out.write_text("".join(path + "\n" for path in unique))
print("Stage E candidate mutation surface accepted:")
for path in unique:
    print(f"  {path}")
PY

sha256sum "$VERIFIER_PATH" | tee "$OUT_DIR/trusted-verifier.sha256"
printf '%s\n' "$TASK_KIND" | tee "$OUT_DIR/task-kind.txt"
git -C "$TRUSTED_ROOT" rev-parse HEAD | tee "$OUT_DIR/trusted-head.txt"

WORKTREE_BASE="$(mktemp -d "${TMPDIR:-/tmp}/viewwitness-m5-stage-e.XXXXXX")"
WORKTREE="$WORKTREE_BASE/candidate"
worktree_registered=0
cleanup() {
  if [[ "$worktree_registered" -eq 1 ]]; then
    git -C "$TRUSTED_ROOT" worktree remove --force "$WORKTREE" >/dev/null 2>&1 || true
  fi
  rm -rf "$WORKTREE_BASE" >/dev/null 2>&1 || true
}
trap cleanup EXIT

# Candidate verification always begins from trusted committed HEAD, never from
# the coding agent's mutable checkout or the caller's current working-tree state.
git -C "$TRUSTED_ROOT" worktree add --detach "$WORKTREE" HEAD
worktree_registered=1

(
  cd "$WORKTREE"
  git apply --check "$PATCH_FILE"
  git apply "$PATCH_FILE"
  git diff --check
  git diff --binary >"$OUT_DIR/applied-candidate.patch"
)

# The verifier is invoked from the trusted checkout through bash so verification
# does not depend on the repository file's executable bit. It receives the fresh
# candidate worktree only as the application build root.
bash "$VERIFIER_PATH" "$BASELINE_DIR" "$OUT_DIR" "$WORKTREE"

echo "M5 trusted agent-patch verification succeeded: kind=$TASK_KIND"
