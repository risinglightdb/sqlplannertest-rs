use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use console::style;
use libtest_mimic::{Arguments, Failed, Trial};
use similar::{ChangeTag, TextDiff};
use tokio::runtime::Runtime;

use crate::apply::generate_result;
use crate::{
    discover_tests, parse_test_cases, PlannerTestRunner, TestCase, RESULT_SUFFIX, TEST_SUFFIX,
};

// Copyright 2022 Armin Ronacher
struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}
// End Copyright 2022 Armin Ronacher

/// Test runner based on libtest-mimic.
pub fn planner_test_runner<F, Ft, R>(path: impl AsRef<Path>, runner_fn: F) -> Result<()>
where
    F: Fn() -> Ft + Send + Sync + 'static + Clone,
    Ft: Future<Output = Result<R>> + Send,
    R: PlannerTestRunner,
{
    let paths = discover_tests(path)?;

    let args = Arguments::from_args();

    let mut tests = vec![];
    for entry in paths {
        let path = entry.context("failed to read glob entry")?;
        let filename = path.file_name().context("unable to extract filename")?;
        let testname = filename.to_str().context("unable to convert to string")?;

        let nocapture = args.nocapture;
        let runner_fn = runner_fn.clone();

        tests.push(Trial::test(
            testname
                .strip_suffix(TEST_SUFFIX)
                .unwrap()
                .replace('/', "_"),
            move || run(path, nocapture, runner_fn),
        ));
    }

    if tests.is_empty() {
        return Err(anyhow!("no test discovered"));
    }

    libtest_mimic::run(&args, tests).exit();
}

fn run<F, Ft, R>(path: PathBuf, nocapture: bool, runner_fn: F) -> Result<(), Failed>
where
    F: Fn() -> Ft + Send + Sync + 'static + Clone,
    Ft: Future<Output = Result<R>> + Send,
    R: PlannerTestRunner,
{
    fn build_runtime() -> Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    build_runtime().block_on(async move {
        let mut runner = runner_fn().await?;
        let testcases = tokio::fs::read(&path).await?;
        let testcases: Vec<TestCase> = serde_yaml::from_slice(&testcases)?;
        let testcases = parse_test_cases(
            {
                let mut path = path.clone();
                path.pop();
                path
            },
            testcases,
        )?;
        let mut generated_result = String::new();
        for testcase in testcases {
            let runner_result = runner.run(&testcase).await;
            if !testcase.no_capture {
                generate_result(&testcase, &runner_result, &mut generated_result)?;
            }
        }
        let path = {
            let mut path = path;
            path.set_extension(RESULT_SUFFIX);
            path
        };
        let expected_result = tokio::fs::read_to_string(&path).await?;

        let diff = TextDiff::from_lines(&generated_result, &expected_result);

        for change in diff.iter_all_changes() {
            use console::Style;
            let (sign, sty) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
            };

            if nocapture {
                print!(
                    "{}{} {}{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    sty.apply_to(sign).bold(),
                    sty.apply_to(change)
                );
            }
        }

        if generated_result != expected_result {
            Err(Failed::without_message())
        } else {
            Ok(())
        }
    })
}
