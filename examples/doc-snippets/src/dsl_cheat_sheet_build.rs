use testing_framework_core::scenario::{Scenario, ScenarioBuilder};
use testing_framework_workflows::ScenarioBuilderExt;

use crate::SnippetResult;

pub fn build_plan() -> SnippetResult<Scenario<()>> {
    ScenarioBuilder::topology_with(|t| t.network_star().validators(1).executors(0)).build() // Construct the final Scenario
}
