// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Context, Result};

use crate::{ParsedTestCase, TestCase};

pub fn resolve_testcase_id(
    base_path: impl AsRef<Path>,
    testcases: Vec<TestCase>,
) -> Result<Vec<ParsedTestCase>> {
    let mut testcases_with_ids = HashMap::new();
    for testcase in &testcases {
        if let Some(id) = &testcase.id {
            testcases_with_ids.insert(id.clone(), testcase.clone());
        }
    }

    testcases
        .into_iter()
        .map(|testcase| {
            let before_sql = if let Some(before) = &testcase.before {
                Some(
                    before
                        .iter()
                        .map(|maybe_sql| {
                            if let Some(id) = maybe_sql.strip_prefix('*') {
                                testcases_with_ids
                                    .get(id)
                                    .map(|case| case.sql.clone())
                                    .ok_or_else(|| anyhow!("failed to resolve {}: not found", id))
                            } else if let Some(include) = maybe_sql.strip_prefix("include_sql:") {
                                let path = base_path.as_ref().join(include);
                                std::fs::read_to_string(&path)
                                    .with_context(|| format!("failed to read: {}", include))
                            } else {
                                Ok(maybe_sql.to_string())
                            }
                        })
                        .collect::<Result<Vec<_>>>()?,
                )
            } else {
                None
            };

            Ok(ParsedTestCase {
                before_sql: before_sql.unwrap_or_default(),
                id: testcase.id,
                desc: testcase.desc,
                sql: testcase.sql,
                no_capture: testcase.no_capture == Some(true),
                tasks: testcase.tasks.unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>>>()
}
