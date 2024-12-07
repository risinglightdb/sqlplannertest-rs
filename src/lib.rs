mod apply;
mod resolve_id;
mod test_runner;

use std::path::Path;

use anyhow::{Context, Result};
pub use apply::{planner_test_apply, planner_test_apply_with_options, PlannerTestApplyOptions};
use async_trait::async_trait;
use glob::Paths;
use itertools::Itertools;
use resolve_id::resolve_testcase_id;
use serde::{Deserialize, Serialize};
pub use test_runner::planner_test_runner;

/// Describing a test case.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub id: Option<String>,
    pub desc: Option<String>,
    pub sql: String,
    pub before: Option<Vec<String>>,
    pub no_capture: Option<bool>,
    pub tasks: Option<Vec<String>>,
}

/// A parsed test case.
#[derive(Clone, Debug)]
pub struct ParsedTestCase {
    pub id: Option<String>,
    pub desc: Option<String>,
    pub sql: String,
    pub before_sql: Vec<String>,
    pub no_capture: bool,
    pub tasks: Vec<String>,
}

/// A planner test runner.
#[async_trait]
pub trait PlannerTestRunner: Send {
    /// Run a test case and return the result
    async fn run(&mut self, test_case: &ParsedTestCase) -> Result<String>;
}

pub fn parse_test_cases(
    base_path: impl AsRef<Path>,
    tests: Vec<TestCase>,
) -> Result<Vec<ParsedTestCase>> {
    resolve_testcase_id(base_path, tests)
}

const TEST_SUFFIX: &str = ".yml";
const RESULT_SUFFIX: &str = "planner.sql";

pub fn discover_tests(path: impl AsRef<Path>) -> Result<impl Iterator<Item = glob::GlobResult>> {
    discover_tests_with_selections(path, &[])
}

pub fn discover_tests_with_selections(
    path: impl AsRef<Path>,
    selections: &[String],
) -> Result<impl Iterator<Item = glob::GlobResult>> {
    let patterns = mk_patterns(&path, selections);
    let paths: Vec<Paths> = patterns
        .into_iter()
        .map(|pattern| glob::glob(&pattern).context("input pattern is invalid"))
        .try_collect()?;

    Ok(paths.into_iter().flatten())
}

/// Make glob patterns based on `selections`.
///
/// If `selections` is empty, returns the glob pattern that select all tests within `path`.
/// Otherwise returns the glob pattterns that matches the selected test modules or files.
fn mk_patterns(path: impl AsRef<Path>, selections: &[String]) -> Vec<String> {
    let mk_pattern = |glob: String| {
        let path = path.as_ref().join(glob);
        path.to_str().expect("non utf-8 path").to_string()
    };

    if selections.is_empty() {
        // Select all tests
        return vec![mk_pattern(format!("**/[!_]*{}", TEST_SUFFIX))];
    }

    // Select matching tests.
    selections
        .iter()
        .map(|s| {
            let path_segment = s.replace("::", "/");
            // e.g. tests/<..>/path_segment.yml
            let file_match = mk_pattern(format!("**/{path_segment}{}", TEST_SUFFIX));
            // Module match, needs to start at the top level.
            // e.g. tests/path_segment/<..>/<some>.yml
            let module_match = mk_pattern(format!("{path_segment}/**/[!_]*{}", TEST_SUFFIX));
            std::iter::once(file_match).chain(std::iter::once(module_match))
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_planner_test() {
        let x = r#"
- id: sql1
  sql: |
    CREATE TABLE t1(v1 int);
- id: sql2
  sql: |
    CREATE TABLE t2(v2 int);
- sql: |
    SELECT * FROM t1, t2 WHERE t1.v1 = t2.v2;
  desc: "Test whether join works correctly."
  before: ["*sql1", "*sql2", "CREATE TABLE t3(v3 int);"]
  tasks:
    - logical
    - physical
        "#
        .trim();
        let test_case: Vec<TestCase> = serde_yaml::from_str(x).unwrap();
        let test_case = parse_test_cases("", test_case).unwrap();
        assert_eq!(test_case.len(), 3);
        assert_eq!(test_case[2].before_sql.len(), 3);
        assert_eq!(
            test_case[2].before_sql[0].trim(),
            "CREATE TABLE t1(v1 int);"
        );
    }
}
