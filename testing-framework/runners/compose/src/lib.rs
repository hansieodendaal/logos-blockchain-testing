mod block_feed;
mod cfgsync;
mod cleanup;
mod compose;
mod control;
mod deployer;
mod docker;
mod environment;
mod errors;
mod ports;
mod readiness;
mod wait;
mod workspace;

/// The Docker Compose runner entry point.
pub use deployer::ComposeDeployer;
/// Port binding reservation used while wiring Prometheus.
pub use environment::PortReservation;
/// Error types surfaced by the compose runner.
pub use errors::{
    ComposeRunnerError, ConfigError, NodeClientError, StackReadinessError, WorkspaceError,
};
pub use workspace::ComposeWorkspace;
