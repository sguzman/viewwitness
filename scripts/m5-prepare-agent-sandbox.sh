#!/usr/bin/env bash
set -euo pipefail

BASELINE_DIR="${1:-target/m5-separated/baseline}"
SANDBOX_ROOT="${2:-target/m5-stage-e/agent-task}"
TRUSTED_ROOT="$(pwd -P)"
TASK_SOURCE="$TRUSTED_ROOT/prompts/m5-handle-repair.md"

BASELINE_DIR="$(realpath "$BASELINE_DIR")"
SANDBOX_ROOT="$(realpath -m "$SANDBOX_ROOT")"
WORKSPACE="$SANDBOX_ROOT/workspace"
EVIDENCE="$SANDBOX_ROOT/evidence"

for required in \
  "$BASELINE_DIR/broken.yaml" \
  "$BASELINE_DIR/handle-focus.txt" \
  "$BASELINE_DIR/outline-focus.txt" \
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
git -C "$TRUSTED_ROOT" archive HEAD Cargo.toml Cargo.lock src examples \
  | tar -x -C "$WORKSPACE"

cp "$BASELINE_DIR/broken.yaml" "$EVIDENCE/broken.yaml"
cp "$BASELINE_DIR/handle-focus.txt" "$EVIDENCE/handle-focus.txt"
cp "$BASELINE_DIR/outline-focus.txt" "$EVIDENCE/outline-focus.txt"
cp "$TASK_SOURCE" "$SANDBOX_ROOT/TASK.md"

git -C "$TRUSTED_ROOT" rev-parse HEAD >"$SANDBOX_ROOT/trusted-head.txt"

(
  cd "$WORKSPACE"
  git init -q
  git config user.name "ViewWitness Stage E"
  git config user.email "stage-e@viewwitness.invalid"
  git add Cargo.toml Cargo.lock src examples
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

if find "$WORKSPACE" -path '*/m5-reference-handle-patcher.sh' -o -path '*/m5-verify-handle-candidate.sh' | grep -q .; then
  echo "sanitized workspace leaked M5 patcher/verifier tooling" >&2
  exit 1
fi

# Ensure the packaged application is independently buildable before any coding
# agent receives it.
(
  cd "$WORKSPACE"
  cargo check --example showcase --features showcase
)

echo "M5 recipe-free coding-agent sandbox prepared: $SANDBOX_ROOT"
