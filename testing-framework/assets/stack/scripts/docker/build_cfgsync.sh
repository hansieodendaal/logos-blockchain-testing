#!/usr/bin/env bash
set -euo pipefail

RUSTFLAGS='--cfg feature="pol-dev-mode"' \
  cargo build --all-features --manifest-path /workspace/testing-framework/tools/cfgsync_tf/Cargo.toml --bins

cp /workspace/target/debug/cfgsync-server /workspace/artifacts/cfgsync-server
cp /workspace/target/debug/cfgsync-client /workspace/artifacts/cfgsync-client

rm -rf /workspace/target/debug/incremental
