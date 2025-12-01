# Advanced Examples

Realistic advanced scenarios demonstrating framework capabilities for production testing.

## Summary

| Example | Topology | Workloads | Deployer | Key Feature |
|---------|----------|-----------|----------|-------------|
| Load Progression | 3 validators + 2 executors | Increasing tx rate | Compose | Dynamic load testing |
| Sustained Load | 4 validators + 2 executors | High tx + DA rate | Compose | Stress testing |
| Aggressive Chaos | 4 validators + 2 executors | Frequent restarts + traffic | Compose | Resilience validation |

## Load Progression Test

Test consensus under progressively increasing transaction load:

```rust
use testing_framework_core::scenario::{Deployer, ScenarioBuilder};
use testing_framework_runner_compose::ComposeDeployer;
use testing_framework_workflows::ScenarioBuilderExt;
use std::time::Duration;

async fn load_progression_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for rate in [5, 10, 20, 30] {
        println!("Testing with rate: {}", rate);
        
        let mut plan = ScenarioBuilder::topology()
                .network_star()
                .validators(3)
                .executors(2)
                .apply()
            .wallets(50)
            .transactions()
                .rate(rate)
                .users(20)
                .apply()
            .expect_consensus_liveness()
            .with_run_duration(Duration::from_secs(60))
            .build();

        let deployer = ComposeDeployer::default();
        let runner = deployer.deploy(&plan).await?;
        let _handle = runner.run(&mut plan).await?;
    }
    
    Ok(())
}
```

**When to use:** Finding the maximum sustainable transaction rate for a given topology.

## Sustained Load Test

Run high transaction and DA load for extended duration:

```rust
use testing_framework_core::scenario::{Deployer, ScenarioBuilder};
use testing_framework_runner_compose::ComposeDeployer;
use testing_framework_workflows::ScenarioBuilderExt;
use std::time::Duration;

async fn sustained_load_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut plan = ScenarioBuilder::topology()
            .network_star()
            .validators(4)
            .executors(2)
            .apply()
        .wallets(100)
        .transactions()
            .rate(15)
            .users(50)
            .apply()
        .da()
            .channel_rate(2)
            .blob_rate(3)
            .apply()
        .expect_consensus_liveness()
        .with_run_duration(Duration::from_secs(300))
        .build();

    let deployer = ComposeDeployer::default();
    let runner = deployer.deploy(&plan).await?;
    let _handle = runner.run(&mut plan).await?;
    
    Ok(())
}
```

**When to use:** Validating stability under continuous high load over extended periods.

## Aggressive Chaos Test

Frequent node restarts with active traffic:

```rust
use testing_framework_core::scenario::{Deployer, ScenarioBuilder};
use testing_framework_runner_compose::ComposeDeployer;
use testing_framework_workflows::{ScenarioBuilderExt, ChaosBuilderExt};
use std::time::Duration;

async fn aggressive_chaos_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut plan = ScenarioBuilder::topology()
            .network_star()
            .validators(4)
            .executors(2)
            .apply()
        .enable_node_control()
        .wallets(50)
        .transactions()
            .rate(10)
            .users(20)
            .apply()
        .chaos()
            .restart()
            .min_delay(Duration::from_secs(10))
            .max_delay(Duration::from_secs(20))
            .target_cooldown(Duration::from_secs(15))
            .apply()
        .expect_consensus_liveness()
        .with_run_duration(Duration::from_secs(180))
        .build();

    let deployer = ComposeDeployer::default();
    let runner = deployer.deploy(&plan).await?;
    let _handle = runner.run(&mut plan).await?;
    
    Ok(())
}
```

**When to use:** Validating recovery and liveness under aggressive failure conditions.

**Note:** Requires `ComposeDeployer` for node control support.

## Extension Ideas

These scenarios require custom implementations but demonstrate framework extensibility:

### Network Partition Recovery

**Concept:** Test consensus recovery after network partitions.

**Requirements:**
- Needs `block_peer()` / `unblock_peer()` methods in `NodeControlHandle`
- Partition subsets of validators, wait, then restore connectivity
- Verify chain convergence after partition heals

**Why useful:** Tests the most realistic failure mode in distributed systems.

**Current blocker:** Node control doesn't yet support network-level actions (only process restarts).

### Block Timing Consistency

**Concept:** Verify block production intervals stay within expected bounds.

**Implementation approach:**
- Custom expectation that consumes `BlockFeed`
- Collect block timestamps during run
- Assert intervals are within `(slot_duration * active_slot_coeff) ± tolerance`

**Why useful:** Validates consensus timing under various loads.

### Invalid Transaction Fuzzing

**Concept:** Submit malformed transactions and verify they're rejected properly.

**Implementation approach:**
- Custom workload that generates invalid transactions (bad signatures, insufficient funds, malformed structure)
- Expectation verifies mempool rejects them and they never appear in blocks
- Test mempool resilience and filtering

**Why useful:** Ensures mempool doesn't crash or include invalid transactions under fuzzing.

### Wallet Balance Verification

**Concept:** Track wallet balances and verify state consistency.

**Description:** After transaction workload completes, query all wallet balances via node API and verify total supply is conserved. Requires tracking initial state, submitted transactions, and final balances. Validates that the ledger maintains correctness under load (no funds lost or created). This is a **state assertion** expectation that checks correctness, not just liveness.
