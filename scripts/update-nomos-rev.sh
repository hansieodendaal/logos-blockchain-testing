#!/usr/bin/env bash
set -euo pipefail

# Update nomos-node source across versions.env and Cargo.toml.
# Usage:
#   scripts/update-nomos-rev.sh --rev <git_rev>
#   scripts/update-nomos-rev.sh --path <local_dir>
#
# Only one of --rev/--path may be supplied.

usage() {
  cat <<'EOF'
Usage:
  scripts/update-nomos-rev.sh --rev <git_rev>
  scripts/update-nomos-rev.sh --path <local_dir>
  scripts/update-nomos-rev.sh --unskip-worktree

Notes:
  --rev   sets NOMOS_NODE_REV and updates Cargo.toml revs
  --path  sets NOMOS_NODE_PATH (clears NOMOS_NODE_REV) and patches Cargo.toml to use a local nomos-node checkout
  --unskip-worktree clears any skip-worktree flag for Cargo.toml
  Only one may be used at a time.
EOF
}

REV=""
LOCAL_PATH=""
UNSKIP_WORKTREE=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --rev) REV="${2:-}"; shift 2 ;;
    --path) LOCAL_PATH="${2:-}"; shift 2 ;;
    --unskip-worktree) UNSKIP_WORKTREE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

if [ "${UNSKIP_WORKTREE}" -eq 1 ] && { [ -n "${REV}" ] || [ -n "${LOCAL_PATH}" ]; }; then
  echo "Use --unskip-worktree alone." >&2
  usage
  exit 1
fi

if [ -n "${REV}" ] && [ -n "${LOCAL_PATH}" ]; then
  echo "Use either --rev or --path, not both" >&2
  usage; exit 1
fi

if [ -z "${REV}" ] && [ -z "${LOCAL_PATH}" ] && [ "${UNSKIP_WORKTREE}" -eq 0 ]; then
  usage; exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ ! -f "${ROOT_DIR}/versions.env" ]; then
  echo "ERROR: versions.env missing; run from repo root." >&2
  exit 1
fi

# Ensure keys exist so sed replacements succeed even if they were absent.
ensure_env_key() {
  local key="$1" default_value="$2"
  if ! grep -Eq "^#?[[:space:]]*${key}=" "${ROOT_DIR}/versions.env"; then
    echo "${default_value}" >> "${ROOT_DIR}/versions.env"
  fi
}
ensure_env_key "NOMOS_NODE_REV" "# NOMOS_NODE_REV="
ensure_env_key "NOMOS_NODE_PATH" "# NOMOS_NODE_PATH="

maybe_unskip_worktree() {
  local file="$1"
  if git -C "${ROOT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "${ROOT_DIR}" update-index --no-skip-worktree "${file}" >/dev/null 2>&1 || true
  fi
}

maybe_skip_worktree() {
  local file="$1"
  if git -C "${ROOT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "${ROOT_DIR}" update-index --skip-worktree "${file}" >/dev/null 2>&1 || true
  fi
}

if [ "${UNSKIP_WORKTREE}" -eq 1 ]; then
  maybe_unskip_worktree "Cargo.toml"
  echo "Cleared skip-worktree on Cargo.toml (if it was set)."
  exit 0
fi

if [ -n "${REV}" ]; then
  echo "Updating nomos-node rev to ${REV}"
  # Update versions.env NOMOS_NODE_REV entry, clear NOMOS_NODE_PATH if present.
  sed -i.bak -E \
    -e "s/^#?[[:space:]]*NOMOS_NODE_REV=.*/NOMOS_NODE_REV=${REV}/" \
    -e "s/^#?[[:space:]]*NOMOS_NODE_PATH=.*/# NOMOS_NODE_PATH=/" \
    "${ROOT_DIR}/versions.env"
  rm -f "${ROOT_DIR}/versions.env.bak"

  # Update all rev fields in Cargo.toml pointing to nomos-node.
  python3 - "$ROOT_DIR" "$REV" <<'PY'
import pathlib, re, sys
root = pathlib.Path(sys.argv[1])
rev = sys.argv[2]
cargo_toml = root / "Cargo.toml"
txt = cargo_toml.read_text()
txt = txt.replace("\\n", "\n")
txt = re.sub(
    r'(?ms)^\[patch\."https://github\.com/logos-co/nomos-node"\].*?(?=^\[|\Z)',
    "",
    txt,
)
txt = re.sub(
    r'(git = "https://github\.com/logos-co/nomos-node\.git", rev = ")[^"]+(")',
    r"\g<1>" + rev + r"\2",
    txt,
)
cargo_toml.write_text(txt.rstrip() + "\n")
PY
  maybe_unskip_worktree "Cargo.toml"
else
  echo "Pointing to local nomos-node at ${LOCAL_PATH}"
  if [ ! -d "${LOCAL_PATH}" ]; then
    echo "ERROR: path does not exist: ${LOCAL_PATH}" >&2
    exit 1
  fi
  CURRENT_REV="$(grep -E '^[#[:space:]]*NOMOS_NODE_REV=' "${ROOT_DIR}/versions.env" | head -n1 | sed -E 's/^#?[[:space:]]*NOMOS_NODE_REV=//')"
  # Update versions.env to favor the local path.
  sed -i.bak -E \
    -e "s/^#?[[:space:]]*NOMOS_NODE_PATH=.*/NOMOS_NODE_PATH=${LOCAL_PATH//\//\\/}/" \
    -e "s/^#?[[:space:]]*NOMOS_NODE_REV=.*/# NOMOS_NODE_REV=${CURRENT_REV}/" \
    "${ROOT_DIR}/versions.env"
  rm -f "${ROOT_DIR}/versions.env.bak"

  # Generate a patch block for all nomos-node crates we depend on.
  PYTHON_BIN="${PYTHON_BIN:-python3}"
  if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
    echo "ERROR: python3 is required to patch Cargo.toml for local paths" >&2
    exit 1
  fi
  "${PYTHON_BIN}" - "$ROOT_DIR" "$LOCAL_PATH" <<'PY'
import json
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(sys.argv[1])
node_path = pathlib.Path(sys.argv[2])

targets = [
    "broadcast-service", "chain-leader", "chain-network", "chain-service",
    "common-http-client", "cryptarchia-engine", "cryptarchia-sync",
    "executor-http-client", "groth16", "key-management-system-service",
    "kzgrs", "kzgrs-backend", "nomos-api", "nomos-blend-message",
    "nomos-blend-service", "nomos-core", "nomos-da-dispersal",
    "nomos-da-network-core", "nomos-da-network-service", "nomos-da-sampling",
    "nomos-da-verifier", "nomos-executor", "nomos-http-api-common",
    "nomos-ledger", "nomos-libp2p", "nomos-network", "nomos-node",
    "nomos-sdp", "nomos-time", "nomos-tracing", "nomos-tracing-service",
    "nomos-utils", "nomos-wallet", "poc", "pol", "subnetworks-assignations",
    "tests", "tx-service", "wallet", "zksign",
]

try:
    meta = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=node_path,
    )
except subprocess.CalledProcessError as exc:
    sys.stderr.write(f"Failed to run cargo metadata in {node_path}: {exc}\n")
    sys.exit(1)

data = json.loads(meta)
paths = {}
for pkg in data.get("packages", []):
    paths[pkg["name"]] = str(pathlib.Path(pkg["manifest_path"]).parent)

patch_lines = ['[patch."https://github.com/logos-co/nomos-node"]']
missing = []
for name in targets:
    if name in paths:
        patch_lines.append(f'{name} = {{ path = "{paths[name]}" }}')
    else:
        missing.append(name)

cargo_toml = root / "Cargo.toml"
txt = cargo_toml.read_text()
txt = txt.replace("\\n", "\n")
txt = re.sub(
    r'(?ms)^\[patch\."https://github\.com/logos-co/nomos-node"\].*?(?=^\[|\Z)',
    "",
    txt,
)
txt = txt.rstrip() + "\n\n" + "\n".join(patch_lines) + "\n"
cargo_toml.write_text(txt)

if missing:
    sys.stderr.write(
        "Warning: missing crates in local nomos-node checkout: "
        + ", ".join(missing)
        + "\n"
    )
PY

  # Avoid accidentally committing the local patch block.
  maybe_skip_worktree "Cargo.toml"
  echo "Local nomos-node patch applied; Cargo.toml marked skip-worktree (run --unskip-worktree to clear)."
fi

echo "Done. Consider updating Cargo.lock if needed (cargo fetch)."
