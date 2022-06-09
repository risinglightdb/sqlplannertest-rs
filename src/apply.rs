use std::future::Future;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::{discover_tests, parse_test_cases, ParsedTestCase, PlannerTestRunner, TestCase};

pub async fn planner_test_apply<F, Ft, R>(path: impl AsRef<Path>, runner_fn: F) -> Result<()>
where
    F: Fn() -> Ft + Send + Sync + 'static,
    Ft: Future<Output = Result<R>> + Send,
    R: PlannerTestRunner,
{
    let tests = discover_tests(path)?;
    let mut test_discovered = false;
    for entry in tests {
        test_discovered = true;
        let path = entry.context("failed to read glob entry")?;
        let filename = path.file_name().context("unable to extract filename")?;
        let testname = filename.to_str().context("unable to convert to string")?;
        let mut runner = runner_fn().await?;
        let testcases = tokio::fs::read(&path).await?;
        let testcases: Vec<TestCase> = serde_yaml::from_slice(&testcases)?;
        let testcases = parse_test_cases(testcases)?;
        let mut generated_result = String::new();
        println!("{}", testname);
        for testcase in testcases {
            let runner_result = runner.run(&testcase).await?;
            generate_result(&testcase, &runner_result, &mut generated_result)?;
        }
        let path = {
            let mut path = path;
            path.set_extension("sql");
            path
        };
        tokio::fs::write(&path, generated_result).await?;
    }
    if !test_discovered {
        return Err(anyhow!("no test discovered"));
    }
    Ok(())
}

/// Generate a text-based result based on test case and runner result
pub fn generate_result(
    testcase: &ParsedTestCase,
    runner_result: &str,
    mut r: impl std::fmt::Write,
) -> Result<()> {
    writeln!(r, "-----")?;
    writeln!(r, "{}", testcase.sql)?;
    writeln!(r, "{}", runner_result)?;
    writeln!(r)?;
    Ok(())
}
