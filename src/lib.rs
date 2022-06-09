mod apply;
mod resolve_id;
mod test_runner;

use anyhow::Result;
use async_trait::async_trait;
use resolve_id::resolve_testcase_id;
use serde::{Deserialize, Serialize};

pub use apply::planner_test_apply;
pub use test_runner::planner_test_runner;

/// Describing a test case.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub id: Option<String>,
    pub desc: Option<String>,
    pub sql: String,
    pub before: Option<Vec<String>>,
    pub test: Option<Vec<String>>,
}

/// A parsed test case.
#[derive(Clone, Debug)]
pub struct ParsedTestCase {
    pub id: Option<String>,
    pub desc: Option<String>,
    pub before_sql: Vec<String>,
    pub test: Vec<String>,
}

/// A planner test runner.
#[async_trait]
pub trait PlannerTestRunner {
    /// Run a test case and return the result
    async fn run(&mut self, test_case: &ParsedTestCase) -> Result<String>;
}

pub fn parse_test_case(test: Vec<TestCase>) -> Result<Vec<ParsedTestCase>> {
    resolve_testcase_id(test)
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
  test:
    - logical
    - physical
        "#
        .trim();
        let test_case: Vec<TestCase> = serde_yaml::from_str(x).unwrap();
        let test_case = parse_test_case(test_case).unwrap();
        assert_eq!(test_case.len(), 3);
        assert_eq!(test_case[2].before_sql.len(), 3);
        assert_eq!(
            test_case[2].before_sql[0].trim(),
            "CREATE TABLE t1(v1 int);"
        );
    }
}
