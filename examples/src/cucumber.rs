use cucumber::World;
use cucumber_ext::TestingFrameworkWorld;

pub use crate::defaults::Mode;

const FEATURES_PATH: &str = "examples/cucumber/features";

pub use crate::defaults::{init_logging_defaults, init_node_log_dir_defaults, init_tracing};

fn is_compose(
    feature: &cucumber::gherkin::Feature,
    scenario: &cucumber::gherkin::Scenario,
) -> bool {
    scenario.tags.iter().any(|tag| tag == "compose")
        || feature.tags.iter().any(|tag| tag == "compose")
}

pub async fn run(mode: Mode) {
    TestingFrameworkWorld::cucumber()
        .with_default_cli()
        .max_concurrent_scenarios(Some(1))
        .filter_run(FEATURES_PATH, move |feature, _, scenario| match mode {
            Mode::Host => !is_compose(feature, scenario),
            Mode::Compose => is_compose(feature, scenario),
        })
        .await;
}
