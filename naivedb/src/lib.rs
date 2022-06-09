use anyhow::Result;
use async_trait::async_trait;

#[derive(Default)]
pub struct NaiveDb {}

impl NaiveDb {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl sqlplannertest::PlannerTestRunner for NaiveDb {
    async fn run(&mut self, _test_case: &sqlplannertest::ParsedTestCase) -> Result<String> {
        Ok("hello, world!".to_string())
    }
}
