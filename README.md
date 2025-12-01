# Nomos Testing

This repo is the standalone Nomos testing framework. For docs, quick start, and examples, read the mdBook at https://logos-co.github.io/nomos-testing/ (sources in `book/`) — start with:
- What you’ll learn: https://logos-co.github.io/nomos-testing/what-you-will-learn.html
- Quick examples: https://logos-co.github.io/nomos-testing/examples.html and https://logos-co.github.io/nomos-testing/examples-advanced.html
- Runners (compose/k8s/local): https://logos-co.github.io/nomos-testing/runners.html

Key crates live under `testing-framework/` (core, runners, workflows, configs) with integration tests in `tests/workflows/`. Compose/k8s assets sit in `testing-framework/assets/stack/`.
