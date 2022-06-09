# SQLPlannerTest

A yaml-based SQL planner test framework.

SQLPlannerTest is a regression test framework. It will read a special yaml file for describing
the test cases, call the database system to generate result, and store the result in a special
regression test file format.

Here's an example of test description file:

```yaml
- id: sql1
  sql: |
    CREATE TABLE t1(v1 int);
- id: sql2
  sql: |
    CREATE TABLE t2(v2 int);
- sql: |
    SELECT * FROM t1, t2 WHERE t1.v1 = t2.v2;
  desc: Test whether join works correctly.
  before: ["*sql1", "*sql2", "CREATE TABLE t3(v3 int);"]
  tasks:
  - logical
  - physical
```

Basically, it is like:

```yaml
- id: <test case id> # which will be referenced by `before`
  sql: SELECT * FROM table;
  desc: Description of this test case
  before:
  - "*test_case_1"           # use *id to reference to another test case
  - "*test_case_2"
  - CREATE TABLE t2(v2 int); # or directly write a SQL here
  tasks:                      # run logical and physical test for this case
  - logical
  - physical
```

And it will generate a file describing the result. Developers can diff the regression
test result to see what have been changed throughout the process.

## NaiveDB

The naive database system for testing sqlplannertest.

**Update the test cases**

```
cargo run -p naivedb --bin apply
```

**Verify the test cases**

```
cargo test -p naivedb
# or use nextest
cargo nextest run -p naivedb
```
