use std::future::Future;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Error, Result};
use console::style;
use futures_util::{stream, StreamExt, TryFutureExt};

use crate::{
    discover_tests_with_selections, parse_test_cases, ParsedTestCase, PlannerTestRunner, TestCase,
    RESULT_SUFFIX,
};

#[derive(Default, Clone)]
pub struct PlannerTestApplyOptions {
    pub serial: bool,
    /// A selection of test modules or files.
    pub selections: Vec<String>,
}

pub async fn planner_test_apply<F, Ft, R>(path: impl AsRef<Path>, runner_fn: F) -> Result<()>
where
    F: Fn() -> Ft + Send + Sync + 'static,
    Ft: Future<Output = Result<R>> + Send,
    R: PlannerTestRunner + 'static,
{
    planner_test_apply_with_options(path, runner_fn, PlannerTestApplyOptions::default()).await
}

pub async fn planner_test_apply_with_options<F, Ft, R>(
    tests_dir: impl AsRef<Path>,
    runner_fn: F,
    options: PlannerTestApplyOptions,
) -> Result<()>
where
    F: Fn() -> Ft + Send + Sync + 'static,
    Ft: Future<Output = Result<R>> + Send,
    R: PlannerTestRunner + 'static,
{
    let tests = discover_tests_with_selections(&tests_dir, &options.selections)?
        .map(|path| {
            let path = path?;
            let relative_path = path
                .strip_prefix(&tests_dir)
                .context("unable to relative path")?
                .as_os_str();
            let testname = relative_path
                .to_str()
                .context("unable to convert to string")?
                .to_string();
            Ok::<_, Error>((path, testname))
        })
        .collect::<Result<Vec<_>, _>>()?;

    struct TestResult {
        testname: String,
        time: Duration,
    }
    let test_stream = stream::iter(tests).map(|(path, testname)| {
        let runner_fn = &runner_fn;
        let testname_x = testname.clone();
        async {
            let mut runner = runner_fn().await?;
            let start = Instant::now();
            tokio::spawn(async move {
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
                tokio::fs::write(&path, generated_result).await?;

                Ok::<_, Error>(())
            })
            .await??;
            let time = start.elapsed();
            Ok::<_, Error>(TestResult { testname, time })
        }
        .map_err(|e| (e, testname_x))
    });

    let mut test_stream = if options.serial {
        test_stream.then(|x| x).boxed()
    } else {
        test_stream
            .buffer_unordered(std::thread::available_parallelism()?.into())
            .boxed()
    };

    let mut test_discovered = false;
    let mut failed_cases = vec![];
    while let Some(item) = test_stream.next().await {
        match item {
            Ok(item) => println!(
                "{} {}, took {} ms",
                style("[DONE]").green().bold(),
                item.testname,
                item.time.as_millis()
            ),
            Err((e, name)) => {
                println!("{} {}: {:#}", style("[FAIL]").red().bold(), name, e);
                failed_cases.push(name);
            }
        }
        test_discovered = true;
    }

    if !test_discovered {
        return Err(anyhow!("no test discovered"));
    }

    if !failed_cases.is_empty() {
        println!("Failed cases: {:#?}", failed_cases);
        return Err(anyhow!("Cannot apply planner test"));
    }

    Ok(())
}

/// Generate a text-based result based on test case and runner result
pub fn generate_result(
    testcase: &ParsedTestCase,
    runner_result: &Result<String>,
    mut r: impl std::fmt::Write,
) -> Result<()> {
    match (&testcase.id, &testcase.desc) {
        (Some(id), Some(desc)) => writeln!(r, "-- {}: {}", id, desc)?,
        (Some(id), None) => writeln!(r, "-- {}", id)?,
        (None, Some(desc)) => writeln!(r, "-- {}", desc)?,
        (None, None) => writeln!(r, "-- (no id or description)")?,
    }
    writeln!(r, "{}", testcase.sql)?;
    match runner_result {
        Ok(runner_result) => {
            writeln!(r, "/*\n{}\n*/", runner_result.trim_end())?;
        }
        Err(err) => {
            writeln!(r, "/*\nError\n{}\n*/", err)?;
        }
    }
    writeln!(r)?;
    Ok(())
}
