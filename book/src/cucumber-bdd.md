# Cucumber/BDD Interface

The Logos testing repo includes a small Cucumber (Gherkin) harness for “smoke” scenarios. It is useful when you want readable acceptance-style checks, but it intentionally exposes a limited surface area compared to Rust scenarios.

---

## What Exists Today

- Step definitions live in `testing-framework/cucumber`.
- The runnable entrypoints are binaries in `examples` (crate `runner-examples`):
  - `cucumber_host` (local/host deployer)
  - `cucumber_compose` (compose deployer)
- Feature files live in `examples/cucumber/features/`.
- Supported deployers: `local` and `compose` (no k8s runner integration in Cucumber yet).

---

## Example Feature (Matches Current Steps)

This is the shape used by the repo’s smoke features:

```gherkin
Feature: Testing Framework - Local Runner

  Scenario: Run a local smoke scenario (tx + DA + liveness)
    Given deployer is "local"
    And topology has 1 validators and 1 executors
    And run duration is 60 seconds
    And wallets total funds is 1000000000 split across 50 users
    And transactions rate is 1 per block
    And data availability channel rate is 1 per block and blob rate is 1 per block
    And expect consensus liveness
    When run scenario
    Then scenario should succeed
```

---

## Running The Smoke Features

Local runner smoke:

```bash
POL_PROOF_DEV_MODE=true \
cargo run -p runner-examples --bin cucumber_host
```

Compose runner smoke:

```bash
POL_PROOF_DEV_MODE=true \
cargo run -p runner-examples --bin cucumber_compose
```

---

## Available Steps (Current)

Topology / runner selection:
- `Given deployer is "local"|"compose"`
- `Given topology has <validators> validators and <executors> executors`

Run configuration:
- `Given run duration is <seconds> seconds`
- `Given wallets total funds is <funds> split across <users> users`

Workloads:
- `Given transactions rate is <rate> per block`
- `Given transactions rate is <rate> per block using <users> users`
- `Given data availability channel rate is <channel_rate> per block and blob rate is <blob_rate> per block`

Expectations:
- `Given expect consensus liveness`
- `Given consensus liveness lag allowance is <blocks>`

Execution + assertion:
- `When run scenario`
- `Then scenario should succeed`

---

## Notes

- The Cucumber harness builds scenarios using the same core + workflow builder APIs as the Rust examples, so the same prerequisites apply (notably `POL_PROOF_DEV_MODE=true` for practical runs).
- If you need more flexibility (custom workloads/expectations, richer checks, node control/chaos), write Rust scenarios instead: see [Examples](examples.md) and [Extending the Framework](extending.md).
