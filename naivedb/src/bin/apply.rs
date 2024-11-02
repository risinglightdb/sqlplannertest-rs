use std::path::Path;

use anyhow::Result;
use sqlplannertest::PlannerTestApplyOptions;

#[tokio::main]
async fn main() -> Result<()> {
    sqlplannertest::planner_test_apply(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"),
        || async { Ok(naivedb::NaiveDb::new()) },
    )
    .await?;
    Ok(())
}
