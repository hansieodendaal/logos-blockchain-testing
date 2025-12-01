use std::process::Command;

// Manually run the local runner binary as a smoke test.
// This spins up real nodes and should be invoked explicitly:
// POL_PROOF_DEV_MODE=true cargo test -p runner-examples --test
// local_runner_bin_smoke -- --ignored --nocapture
#[test]
#[ignore = "runs local_runner binary (~2min) and requires local assets/binaries"]
fn local_runner_bin_smoke() {
    let status = Command::new("cargo")
        .args([
            "run",
            "-p",
            "runner-examples",
            "--bin",
            "local_runner",
            "--",
            "--nocapture",
        ])
        .env("POL_PROOF_DEV_MODE", "true")
        .env("LOCAL_DEMO_RUN_SECS", "120")
        .env("LOCAL_DEMO_VALIDATORS", "1")
        .env("LOCAL_DEMO_EXECUTORS", "1")
        .status()
        .expect("failed to spawn cargo run");

    assert!(status.success(), "local runner binary exited with {status}");
}
