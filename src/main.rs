use clap::Parser;
use spin_runtime_factors::FactorsBuilder;
use spin_trigger::cli::FactorsTriggerCommand;
use trigger_mcp::McpTrigger;

type Command = FactorsTriggerCommand<McpTrigger, FactorsBuilder>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize telemetry with build info from Spin
    spin_telemetry::init(build_info())?;
    
    // Parse command line arguments and run the trigger
    let trigger = Command::parse();
    trigger.run().await
}

/// Returns build information, matching parent Spin process format
fn build_info() -> String {
    let spin_version = env_var("SPIN_VERSION");
    let spin_commit_sha = env_var("SPIN_COMMIT_SHA");
    let spin_commit_date = env_var("SPIN_COMMIT_DATE");
    format!("{spin_version} ({spin_commit_sha} {spin_commit_date})")
}

fn env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| "unknown".to_string())
}