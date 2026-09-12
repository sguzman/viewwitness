#!/usr/bin/env bash
set -euo pipefail

BASELINE_DIR="${1:-target/m5-separated/baseline}"
SANDBOX_ROOT="${2:-target/m5-stage-e/agent-task}"
TASK_KIND="${3:-handle}"
TRUSTED_ROOT="$(pwd -P)"

case "$TASK_KIND" in
  handle)
    TASK_SOURCE="$TRUSTED_ROOT/prompts/m5-handle-repair.md"
    FOCUS_A="handle-focus.txt"
    FOCUS_B="outline-focus.txt"
    ;;
  clip)
    TASK_SOURCE="$TRUSTED_ROOT/prompts/m5-clip-repair.md"
    FOCUS_A="center-focus.txt"
    FOCUS_B="ring-focus.txt"
    ;;
  *)
    echo "unknown Stage E task kind: $TASK_KIND (expected handle or clip)" >&2
    exit 1
    ;;
esac

BASELINE_DIR="$(realpath "$BASELINE_DIR")"
SANDBOX_ROOT="$(realpath -m "$SANDBOX_ROOT")"
WORKSPACE="$SANDBOX_ROOT/workspace"
EVIDENCE="$SANDBOX_ROOT/evidence"

for required in \
  "$BASELINE_DIR/broken.yaml" \
  "$BASELINE_DIR/$FOCUS_A" \
  "$BASELINE_DIR/$FOCUS_B" \
  "$TASK_SOURCE"; do
  if [[ ! -s "$required" ]]; then
    echo "missing Stage E input: $required" >&2
    exit 1
  fi
done

rm -rf "$SANDBOX_ROOT"
mkdir -p "$WORKSPACE" "$EVIDENCE"

# Build the agent workspace from committed HEAD, never from the caller's mutable
# working tree. Historical tests/docs/scripts are intentionally absent because
# they contain prior acceptance answers and repair recipes.
archive_paths=(Cargo.toml src examples)
if git -C "$TRUSTED_ROOT" cat-file -e HEAD:Cargo.lock 2>/dev/null; then
  archive_paths+=(Cargo.lock)
fi

git -C "$TRUSTED_ROOT" archive HEAD "${archive_paths[@]}" \
  | tar -x -C "$WORKSPACE"

cp "$BASELINE_DIR/broken.yaml" "$EVIDENCE/broken.yaml"
cp "$BASELINE_DIR/$FOCUS_A" "$EVIDENCE/$FOCUS_A"
cp "$BASELINE_DIR/$FOCUS_B" "$EVIDENCE/$FOCUS_B"
cp "$TASK_SOURCE" "$SANDBOX_ROOT/TASK.md"

git -C "$TRUSTED_ROOT" rev-parse HEAD >"$SANDBOX_ROOT/trusted-head.txt"
printf '%s\n' "$TASK_KIND" >"$SANDBOX_ROOT/task-kind.txt"

(
  cd "$WORKSPACE"
  git init -q
  git config user.name "ViewWitness Stage E"
  git config user.email "stage-e@viewwitness.invalid"
  git add Cargo.toml src examples
  if [[ -f Cargo.lock ]]; then
    git add Cargo.lock
  fi
  git commit -q -m "Stage E broken application baseline"
)

# Record exactly what the coding agent can see. This also makes accidental answer
# leakage through newly copied directories obvious in artifacts.
(
  cd "$SANDBOX_ROOT"
  find . -path './workspace/.git' -prune -o -type f -print | sort
) | tee "$SANDBOX_ROOT/visible-files.txt"

for forbidden in \
  "$WORKSPACE/tests" \
  "$WORKSPACE/docs" \
  "$WORKSPACE/scripts" \
  "$WORKSPACE/prompts" \
  "$WORKSPACE/.github"; do
  if [[ -e "$forbidden" ]]; then
    echo "sanitized workspace unexpectedly contains trusted/history surface: $forbidden" >&2
    exit 1
  fi
done

if find "$WORKSPACE" -path '*/m5-reference-handle-patcher.sh' \
    -o -path '*/m5-verify-handle-candidate.sh' \
    -o -path '*/m5-verify-clip-candidate.sh' | grep -q .; then
  echo "sanitized workspace leaked M5 patcher/verifier tooling" >&2
  exit 1
fi

# Prove the packaged application is independently buildable without depositing
# compilation intermediates inside the preserved agent/evidence artifact tree.
SANDBOX_CARGO_TARGET="$(mktemp -d "${TMPDIR:-/tmp}/viewwitness-m5-sandbox-target.XXXXXX")"
cleanup_build_target() {
  rm -rf "$SANDBOX_CARGO_TARGET" >/dev/null 2>&1 || true
}
trap cleanup_build_target EXIT

(
  cd "$WORKSPACE"
  CARGO_TARGET_DIR="$SANDBOX_CARGO_TARGET" \
    cargo check --example showcase --features showcase
)

cleanup_build_target
trap - EXIT

echo "M5 recipe-free coding-agent sandbox prepared: kind=$TASK_KIND root=$SANDBOX_ROOT"
