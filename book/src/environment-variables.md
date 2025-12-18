# Environment Variables Reference

Complete reference of environment variables used by the testing framework, organized by category.

## Critical Variables

These MUST be set for successful test runs:

| Variable | Required | Default | Effect |
|----------|----------|---------|--------|
| `POL_PROOF_DEV_MODE` | **YES** | — | **REQUIRED for all runners**. Set to `true` to use fast dev-mode proving instead of expensive Groth16. Without this, tests will hang/timeout. |

**Example:**

```bash
export POL_PROOF_DEV_MODE=true
```

Or add to your shell profile (`~/.bashrc`, `~/.zshrc`):

```bash
# Required for nomos-testing framework
export POL_PROOF_DEV_MODE=true
```

---

## Runner Selection & Topology

Control which runner to use and the test topology:

| Variable | Default | Effect |
|----------|---------|--------|
| `NOMOS_DEMO_VALIDATORS` | 1 | Number of validators (all runners) |
| `NOMOS_DEMO_EXECUTORS` | 1 | Number of executors (all runners) |
| `NOMOS_DEMO_RUN_SECS` | 60 | Run duration in seconds (all runners) |
| `LOCAL_DEMO_VALIDATORS` | — | Legacy: Number of validators (host runner only) |
| `LOCAL_DEMO_EXECUTORS` | — | Legacy: Number of executors (host runner only) |
| `LOCAL_DEMO_RUN_SECS` | — | Legacy: Run duration (host runner only) |
| `COMPOSE_NODE_PAIRS` | — | Compose-specific topology format: "validators×executors" (e.g., `3x2`) |

**Example:**

```bash
# Run with 5 validators, 2 executors, for 120 seconds
NOMOS_DEMO_VALIDATORS=5 \
NOMOS_DEMO_EXECUTORS=2 \
NOMOS_DEMO_RUN_SECS=120 \
scripts/run/run-examples.sh -t 120 -v 5 -e 2 host
```

---

## Node Binaries (Host Runner)

Required for host runner when not using helper scripts:

| Variable | Required | Default | Effect |
|----------|----------|---------|--------|
| `NOMOS_NODE_BIN` | Yes (host) | — | Path to `nomos-node` binary |
| `NOMOS_EXECUTOR_BIN` | Yes (host) | — | Path to `nomos-executor` binary |
| `NOMOS_NODE_PATH` | No | — | Path to nomos-node git checkout (dev workflow) |

**Example:**

```bash
export NOMOS_NODE_BIN=/path/to/nomos-node/target/release/nomos-node
export NOMOS_EXECUTOR_BIN=/path/to/nomos-node/target/release/nomos-executor
```

---

## Docker Images (Compose / K8s)

Required for compose and k8s runners:

| Variable | Required | Default | Effect |
|----------|----------|---------|--------|
| `NOMOS_TESTNET_IMAGE` | Yes (compose/k8s) | `logos-blockchain-testing:local` | Docker image tag for node containers |
| `NOMOS_BINARIES_TAR` | No | — | Path to prebuilt bundle (`.tar.gz`) for image build |
| `NOMOS_SKIP_IMAGE_BUILD` | No | 0 | Skip image rebuild (compose/k8s); assumes image already exists |

**Example:**

```bash
# Using prebuilt bundle
export NOMOS_BINARIES_TAR=.tmp/nomos-binaries-linux-v0.3.1.tar.gz
export NOMOS_TESTNET_IMAGE=logos-blockchain-testing:local
scripts/build/build_test_image.sh

# Using pre-existing image (skip build)
export NOMOS_SKIP_IMAGE_BUILD=1
scripts/run/run-examples.sh -t 60 -v 3 -e 1 compose
```

---

## Circuit Assets (KZG Parameters)

Circuit asset configuration for DA workloads:

| Variable | Default | Effect |
|----------|---------|--------|
| `NOMOS_KZGRS_PARAMS_PATH` | `testing-framework/assets/stack/kzgrs_test_params/kzgrs_test_params` | Path to KZG proving key file |
| `NOMOS_KZG_DIR_REL` | `testing-framework/assets/stack/kzgrs_test_params` | Directory containing KZG assets (relative to workspace root) |
| `VERSION` | From `versions.env` | Circuit release tag (used by helper scripts) |

**Example:**

```bash
# Use custom circuit assets
NOMOS_KZGRS_PARAMS_PATH=/custom/path/to/kzgrs_test_params \
cargo run -p runner-examples --bin local_runner
```

---

## Node Logging

Control node log output (not framework runner logs):

| Variable | Default | Effect |
|----------|---------|--------|
| `NOMOS_LOG_LEVEL` | `info` | Global log level: `error`, `warn`, `info`, `debug`, `trace` |
| `NOMOS_LOG_FILTER` | — | Fine-grained module filtering (e.g., `cryptarchia=trace,nomos_da_sampling=debug`) |
| `NOMOS_LOG_DIR` | — | Host runner: directory for per-node log files (persistent). Compose/k8s: use `cfgsync.yaml` for file logging. |
| `NOMOS_TESTS_KEEP_LOGS` | 0 | Keep per-run temporary directories (useful for debugging/CI artifacts) |
| `NOMOS_TESTS_TRACING` | false | Enable debug tracing preset (combine with `NOMOS_LOG_DIR` unless external tracing backends configured) |

**Important:** Nodes ignore `RUST_LOG` and only respond to `NOMOS_*` variables.

**Example:**

```bash
# Debug logging to files
NOMOS_LOG_DIR=/tmp/test-logs \
NOMOS_LOG_LEVEL=debug \
NOMOS_LOG_FILTER="cryptarchia=trace,nomos_da_sampling=debug" \
POL_PROOF_DEV_MODE=true \
cargo run -p runner-examples --bin local_runner

# Inspect logs
ls /tmp/test-logs/
# nomos-node-0.2024-12-18T14-30-00.log
# nomos-node-1.2024-12-18T14-30-00.log
```

**Common filter targets:**

| Target Prefix | Subsystem |
|---------------|-----------|
| `cryptarchia` | Consensus (Cryptarchia) |
| `nomos_da_sampling` | DA sampling service |
| `nomos_da_dispersal` | DA dispersal service |
| `nomos_da_verifier` | DA verification |
| `nomos_blend` | Mix network/privacy layer |
| `chain_service` | Chain service (node APIs/state) |
| `chain_network` | P2P networking |
| `chain_leader` | Leader election |

---

## Observability & Metrics

Optional observability integration:

| Variable | Default | Effect |
|----------|---------|--------|
| `NOMOS_METRICS_QUERY_URL` | — | Prometheus-compatible base URL for runner to query (e.g., `http://localhost:9090`) |
| `NOMOS_METRICS_OTLP_INGEST_URL` | — | Full OTLP HTTP ingest URL for node metrics export (e.g., `http://localhost:9090/api/v1/otlp/v1/metrics`) |
| `NOMOS_GRAFANA_URL` | — | Grafana base URL for printing/logging (e.g., `http://localhost:3000`) |
| `NOMOS_OTLP_ENDPOINT` | — | OTLP trace endpoint (optional) |
| `NOMOS_OTLP_METRICS_ENDPOINT` | — | OTLP metrics endpoint (optional) |

**Example:**

```bash
# Enable Prometheus querying
export NOMOS_METRICS_QUERY_URL=http://localhost:9090
export NOMOS_METRICS_OTLP_INGEST_URL=http://localhost:9090/api/v1/otlp/v1/metrics
export NOMOS_GRAFANA_URL=http://localhost:3000

scripts/run/run-examples.sh -t 60 -v 3 -e 1 compose
```

---

## Compose Runner Specific

Variables specific to Docker Compose deployment:

| Variable | Default | Effect |
|----------|---------|--------|
| `COMPOSE_RUNNER_HOST` | `127.0.0.1` | Host address for port mappings |
| `COMPOSE_RUNNER_PRESERVE` | 0 | Keep containers running after test (for debugging) |
| `COMPOSE_RUNNER_HTTP_TIMEOUT_SECS` | — | Override HTTP readiness timeout (seconds) |
| `COMPOSE_RUNNER_HOST_GATEWAY` | `host.docker.internal:host-gateway` | Controls `extra_hosts` entry injected into compose (set to `disable` to omit) |
| `TESTNET_RUNNER_PRESERVE` | — | Alias for `COMPOSE_RUNNER_PRESERVE` |

**Example:**

```bash
# Keep containers after test for debugging
COMPOSE_RUNNER_PRESERVE=1 \
scripts/run/run-examples.sh -t 60 -v 3 -e 1 compose

# Containers remain running
docker ps --filter "name=nomos-compose-"
docker logs <container-id>
```

---

## K8s Runner Specific

Variables specific to Kubernetes deployment:

| Variable | Default | Effect |
|----------|---------|--------|
| `K8S_RUNNER_NAMESPACE` | Random UUID | Kubernetes namespace (pin for debugging) |
| `K8S_RUNNER_RELEASE` | Random UUID | Helm release name (pin for debugging) |
| `K8S_RUNNER_NODE_HOST` | — | NodePort host resolution for non-local clusters |
| `K8S_RUNNER_DEBUG` | 0 | Log Helm stdout/stderr for install commands |
| `K8S_RUNNER_PRESERVE` | 0 | Keep namespace/release after run (for debugging) |
| `K8S_RUNNER_DEPLOYMENT_TIMEOUT_SECS` | — | Override deployment readiness timeout |
| `K8S_RUNNER_HTTP_TIMEOUT_SECS` | — | Override HTTP readiness timeout (port-forwards) |
| `K8S_RUNNER_HTTP_PROBE_TIMEOUT_SECS` | — | Override HTTP readiness timeout (NodePort probes) |
| `K8S_RUNNER_PROMETHEUS_HTTP_TIMEOUT_SECS` | — | Override Prometheus readiness timeout |
| `K8S_RUNNER_PROMETHEUS_HTTP_PROBE_TIMEOUT_SECS` | — | Override Prometheus NodePort probe timeout |

**Example:**

```bash
# Pin namespace for debugging
K8S_RUNNER_NAMESPACE=nomos-test-debug \
K8S_RUNNER_PRESERVE=1 \
K8S_RUNNER_DEBUG=1 \
scripts/run/run-examples.sh -t 60 -v 3 -e 1 k8s

# Inspect resources
kubectl get pods -n nomos-test-debug
kubectl logs -n nomos-test-debug -l nomos/logical-role=validator
```

---

## Platform & Build Configuration

Platform-specific build configuration:

| Variable | Default | Effect |
|----------|---------|--------|
| `NOMOS_BUNDLE_DOCKER_PLATFORM` | Host arch | Docker platform for bundle builds: `linux/arm64` or `linux/amd64` (macOS/Windows hosts) |
| `COMPOSE_CIRCUITS_PLATFORM` | Host arch | Circuits platform for image builds: `linux-aarch64` or `linux-x86_64` |

**macOS / Apple Silicon:**

```bash
# Native performance (recommended for local testing)
export NOMOS_BUNDLE_DOCKER_PLATFORM=linux/arm64

# Or target amd64 (slower via emulation)
export NOMOS_BUNDLE_DOCKER_PLATFORM=linux/amd64
```

---

## Timeouts & Performance

Timeout and performance tuning:

| Variable | Default | Effect |
|----------|---------|--------|
| `SLOW_TEST_ENV` | false | Doubles built-in readiness timeouts (useful in CI / constrained laptops) |
| `TESTNET_PRINT_ENDPOINTS` | 0 | Print `TESTNET_ENDPOINTS` / `TESTNET_PPROF` lines during deploy (set automatically by `scripts/run/run-examples.sh`) |

**Example:**

```bash
# Increase timeouts for slow environments
SLOW_TEST_ENV=true \
scripts/run/run-examples.sh -t 120 -v 5 -e 2 compose
```

---

## Node Configuration (Advanced)

Node-level configuration passed through to nomos-node/nomos-executor:

| Variable | Default | Effect |
|----------|---------|--------|
| `CONSENSUS_SLOT_TIME` | — | Consensus slot time (seconds) |
| `CONSENSUS_ACTIVE_SLOT_COEFF` | — | Active slot coefficient (0.0-1.0) |

**Example:**

```bash
# Faster block production
CONSENSUS_SLOT_TIME=5 \
CONSENSUS_ACTIVE_SLOT_COEFF=0.9 \
POL_PROOF_DEV_MODE=true \
cargo run -p runner-examples --bin local_runner
```

---

## Framework Runner Logging (Not Node Logs)

Control framework runner process logs (uses `RUST_LOG`, not `NOMOS_*`):

| Variable | Default | Effect |
|----------|---------|--------|
| `RUST_LOG` | — | Framework runner log level (e.g., `debug`, `info`) |
| `RUST_BACKTRACE` | — | Enable Rust backtraces on panic (`1` or `full`) |
| `CARGO_TERM_COLOR` | — | Cargo output color (`always`, `never`, `auto`) |

**Example:**

```bash
# Debug framework runner (not nodes)
RUST_LOG=debug \
RUST_BACKTRACE=1 \
cargo run -p runner-examples --bin local_runner
```

---

## Helper Script Variables

Variables used by helper scripts (`scripts/run/run-examples.sh`, etc.):

| Variable | Default | Effect |
|----------|---------|--------|
| `NOMOS_NODE_REV` | From `versions.env` | nomos-node git revision to build/fetch |
| `NOMOS_BUNDLE_VERSION` | From `versions.env` | Bundle schema version |

---

## Quick Reference Examples

### Minimal Host Run

```bash
POL_PROOF_DEV_MODE=true \
scripts/run/run-examples.sh -t 60 -v 3 -e 1 host
```

### Debug Logging (Host)

```bash
POL_PROOF_DEV_MODE=true \
NOMOS_LOG_DIR=/tmp/logs \
NOMOS_LOG_LEVEL=debug \
NOMOS_LOG_FILTER="cryptarchia=trace" \
scripts/run/run-examples.sh -t 60 -v 3 -e 1 host
```

### Compose with Observability

```bash
POL_PROOF_DEV_MODE=true \
NOMOS_METRICS_QUERY_URL=http://localhost:9090 \
NOMOS_GRAFANA_URL=http://localhost:3000 \
scripts/run/run-examples.sh -t 60 -v 3 -e 1 compose
```

### K8s with Debug

```bash
POL_PROOF_DEV_MODE=true \
K8S_RUNNER_NAMESPACE=nomos-debug \
K8S_RUNNER_DEBUG=1 \
K8S_RUNNER_PRESERVE=1 \
scripts/run/run-examples.sh -t 60 -v 3 -e 1 k8s
```

### CI Environment

```yaml
env:
  POL_PROOF_DEV_MODE: true
  RUST_BACKTRACE: 1
  NOMOS_TESTS_KEEP_LOGS: 1
```

---

## See Also

- [Prerequisites & Setup](prerequisites.md) — Required files and setup
- [Running Examples](running-examples.md) — How to run scenarios
- [Logging & Observability](logging-observability.md) — Log collection details
- [CI Integration](ci-integration.md) — CI-specific variables
- [Troubleshooting](troubleshooting.md) — Common issues with variables
