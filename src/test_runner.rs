use libtest_mimic::{run_tests, Arguments, Outcome, Test};

/// Test runner based on libtest-mimic.
pub fn planner_test_runner() {
    // Parse command line arguments
    let args = Arguments::from_args();

    // Create a list of tests (in this case: three dummy tests)
    let tests = vec![
        Test::test("toph"),
        Test::test("sokka"),
        Test {
            name: "long_computation".into(),
            kind: "".into(),
            is_ignored: true,
            is_bench: false,
            data: (),
        },
    ];

    run_tests(&args, tests, |_| Outcome::Passed).exit();
}
