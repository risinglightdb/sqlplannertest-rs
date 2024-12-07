use std::path::Path;

use anyhow::Result;
use clap::Parser;
use sqlplannertest::PlannerTestApplyOptions;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional list of selections to apply the test; if empty, apply all tests
    selections: Vec<String>,
    /// Execute tests in serial
    #[clap(long)]
    serial: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let options = PlannerTestApplyOptions {
        serial: cli.serial,
        selections: cli.selections,
    };
    sqlplannertest::planner_test_apply_with_options(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        || async { Ok(naivedb::NaiveDb::new()) },
        options,
    )
    .await?;
    Ok(())
}
