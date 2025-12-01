# Builder API Quick Reference

Quick reference for the scenario builder DSL. All methods are chainable.

## Imports

```rust
use testing_framework_core::scenario::{Deployer, ScenarioBuilder};
use testing_framework_runner_local::LocalDeployer;
use testing_framework_runner_compose::ComposeDeployer;
use testing_framework_runner_k8s::K8sDeployer;
use testing_framework_workflows::{ScenarioBuilderExt, ChaosBuilderExt};
use std::time::Duration;
```

## Topology

```rust
ScenarioBuilder::topology()
        .network_star()      // Star topology (all connect to seed node)
        .validators(3)       // Number of validator nodes
        .executors(2)        // Number of executor nodes
        .apply()             // Finish topology configuration
```

## Wallets

```rust
.wallets(50)                 // Seed 50 funded wallet accounts
```

## Transaction Workload

```rust
.transactions()
    .rate(5)                 // 5 transactions per block
    .users(20)               // Use 20 of the seeded wallets
    .apply()                 // Finish transaction workload config
```

## DA Workload

```rust
.da()
    .channel_rate(1)         // 1 channel operation per block
    .blob_rate(2)            // 2 blob dispersals per block
    .apply()                 // Finish DA workload config
```

## Chaos Workload (Requires `enable_node_control()`)

```rust
.enable_node_control()       // Enable node control capability
.chaos()
    .restart()               // Random restart chaos
    .min_delay(Duration::from_secs(30))     // Min time between restarts
    .max_delay(Duration::from_secs(60))     // Max time between restarts
    .target_cooldown(Duration::from_secs(45))  // Cooldown after restart
    .apply()                 // Finish chaos workload config
```

## Expectations

```rust
.expect_consensus_liveness() // Assert blocks are produced continuously
```

## Run Duration

```rust
.with_run_duration(Duration::from_secs(120))  // Run for 120 seconds
```

## Build

```rust
.build()                     // Construct the final Scenario
```

## Deployers

```rust
// Local processes
let deployer = LocalDeployer::default();

// Docker Compose
let deployer = ComposeDeployer::default();

// Kubernetes
let deployer = K8sDeployer::default();
```

## Execution

```rust
let runner = deployer.deploy(&plan).await?;
let _handle = runner.run(&mut plan).await?;
```

## Complete Example

```rust
use testing_framework_core::scenario::{Deployer, ScenarioBuilder};
use testing_framework_runner_local::LocalDeployer;
use testing_framework_workflows::ScenarioBuilderExt;
use std::time::Duration;

async fn run_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut plan = ScenarioBuilder::topology()
            .network_star()
            .validators(3)
            .executors(2)
            .apply()
        .wallets(50)
        .transactions()
            .rate(5)                     // 5 transactions per block
            .users(20)
            .apply()
        .da()
            .channel_rate(1)             // 1 channel operation per block
            .blob_rate(2)                // 2 blob dispersals per block
            .apply()
        .expect_consensus_liveness()
        .with_run_duration(Duration::from_secs(90))
        .build();

    let deployer = LocalDeployer::default();
    let runner = deployer.deploy(&plan).await?;
    let _handle = runner.run(&mut plan).await?;
    
    Ok(())
}
```
