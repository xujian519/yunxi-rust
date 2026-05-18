mod matching;
mod parser;
#[cfg(test)]
mod tests;
mod types;

pub use matching::IntentRouter;
pub use parser::{identify_scenario_from_input, ScenarioIdentifier};
pub use types::{Domain, Intent, PatentIntent, Phase, ScenarioContext, TaskType};
