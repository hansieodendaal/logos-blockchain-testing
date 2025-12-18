use runner_examples::{
    cucumber::run,
    defaults::{Mode, init_logging_defaults, init_node_log_dir_defaults, init_tracing},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_logging_defaults();
    init_node_log_dir_defaults(Mode::Compose);
    init_tracing();

    run(Mode::Compose).await;
}
