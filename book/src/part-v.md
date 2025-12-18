# Part V — Operations & Deployment

This section covers operational aspects of running the testing framework: prerequisites, deployment configuration, continuous integration, and observability.

## What You'll Learn

- **Prerequisites & Setup**: Required files, binaries, circuit assets, and environment configuration
- **Running Examples**: How to execute scenarios across host, compose, and k8s runners
- **CI Integration**: Automating tests in continuous integration pipelines with caching and matrix testing
- **Environment Variables**: Complete reference of all configuration variables
- **Logging & Observability**: Log collection strategies, metrics integration, and debugging techniques

## Who This Section Is For

- **Operators** setting up the framework for the first time
- **DevOps Engineers** integrating tests into CI/CD pipelines
- **Developers** debugging test failures or performance issues
- **Platform Engineers** deploying across different environments (local, Docker, Kubernetes)

## Navigation

This section is organized for progressive depth:

1. Start with [Operations Overview](operations-overview.md) for the big picture
2. Follow [Prerequisites & Setup](prerequisites.md) to prepare your environment
3. Use [Running Examples](running-examples.md) to execute your first scenarios
4. Integrate with [CI Integration](ci-integration.md) for automated testing
5. Reference [Environment Variables](environment-variables.md) for complete configuration options
6. Debug with [Logging & Observability](logging-observability.md) when issues arise

## Key Principles

**Operational Hygiene:** Assets present, prerequisites satisfied, observability reachable

**Environment Fit:** Choose the right deployment target based on isolation, reproducibility, and resource needs

**Clear Signals:** Verify runners report node readiness before starting workloads

**Failure Triage:** Map failures to specific causes—missing prerequisites, platform issues, or unmet expectations

---

Ready to get started? Begin with [Operations Overview](operations-overview.md) →

