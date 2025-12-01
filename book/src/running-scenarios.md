# Running Scenarios

Running a scenario follows the same conceptual flow regardless of environment:

1. Select or author a scenario plan that pairs a topology with workloads,
   expectations, and a suitable run window.
2. Choose a deployer aligned with your environment (local, compose, or k8s) and
   ensure its prerequisites are available.
3. Deploy the plan through the deployer, which provisions infrastructure and
   returns a runner.
4. The runner orchestrates workload execution for the planned duration; keep
   observability signals visible so you can correlate outcomes.
5. The runner evaluates expectations and captures results as the primary
   pass/fail signal.

Use the same plan across different deployers to compare behavior between local
development and CI or cluster settings. For environment prerequisites and
flags, see [Operations](operations.md).
