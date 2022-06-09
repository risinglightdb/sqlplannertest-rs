use anyhow::{anyhow, Result};
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
    async fn run(&mut self, test_case: &sqlplannertest::ParsedTestCase) -> Result<String> {
        use std::fmt::Write;
        let mut result = String::new();
        let r = &mut result;
        for task in &test_case.tasks {
            writeln!(r, "=== {}", task)?;
            writeln!(r, "I'm a naive db, so I don't now how to process")?;
            for before in &test_case.before_sql {
                writeln!(r, "{}", before)?;
            }
            writeln!(r, "{}", test_case.sql)?;
            writeln!(r)?;
        }
        if test_case.sql.contains("ERROR") {
            return Err(anyhow!("Ooops, error!"));
        }
        if test_case.sql.contains("PANIC") {
            panic!("I'm panic!");
        }
        Ok(result)
    }
}
