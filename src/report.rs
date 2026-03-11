use std::collections::HashSet;
use std::fmt::Write;
use std::time::Duration;

use sqllogictest::TestError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ErrorRecord {
    filename: String,
    query_id: Option<String>,
    detail: String,
}

impl ErrorRecord {
    pub(crate) fn new(
        filename: impl Into<String>,
        error: TestError,
        query_id: Option<String>,
    ) -> Self {
        Self {
            filename: filename.into(),
            query_id,
            detail: error.display(true).to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunStatus {
    NoTestsRun,
    Passed,
    Failed,
}

pub(crate) struct RunReport {
    client_type: String,
    selected_files: usize,
    discovered_tests: usize,
    tests_were_run: bool,
    failed_files: usize,
    no_fail_fast: bool,
    duration: Duration,
    error_records: Vec<ErrorRecord>,
}

impl RunReport {
    pub(crate) fn new(
        client_type: impl Into<String>,
        selected_files: usize,
        discovered_tests: usize,
        tests_were_run: bool,
        no_fail_fast: bool,
        duration: Duration,
        mut error_records: Vec<ErrorRecord>,
    ) -> Self {
        error_records.sort();
        let failed_files = error_records
            .iter()
            .map(|record| record.filename.as_str())
            .collect::<HashSet<_>>()
            .len();

        Self {
            client_type: client_type.into(),
            selected_files,
            discovered_tests,
            tests_were_run,
            failed_files,
            no_fail_fast,
            duration,
            error_records,
        }
    }

    pub(crate) fn render(&self) -> String {
        let mut output = String::new();
        writeln!(&mut output, "Test report for {}", self.client_type).unwrap();
        writeln!(&mut output, "Status: {}", self.status_label()).unwrap();
        writeln!(&mut output, "Files selected: {}", self.selected_files).unwrap();
        writeln!(&mut output, "Tests discovered: {}", self.discovered_tests).unwrap();
        writeln!(&mut output, "Duration: {} ms", self.duration.as_millis()).unwrap();

        if self.status() == RunStatus::Failed {
            writeln!(
                &mut output,
                "Fail fast: {}",
                if self.no_fail_fast {
                    "disabled"
                } else {
                    "enabled"
                }
            )
            .unwrap();
            writeln!(&mut output, "Failed files: {}", self.failed_files).unwrap();
            writeln!(&mut output, "Failed records: {}", self.error_records.len()).unwrap();
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "Failures:").unwrap();
            self.render_failures(&mut output);
        }

        writeln!(&mut output).unwrap();
        write!(&mut output, "Summary: {}", self.summary_line()).unwrap();
        output
    }

    pub(crate) fn has_failures(&self) -> bool {
        !self.error_records.is_empty()
    }

    fn status(&self) -> RunStatus {
        if !self.tests_were_run {
            RunStatus::NoTestsRun
        } else if self.error_records.is_empty() {
            RunStatus::Passed
        } else {
            RunStatus::Failed
        }
    }

    fn status_label(&self) -> &'static str {
        match self.status() {
            RunStatus::NoTestsRun => "NO TESTS RUN",
            RunStatus::Passed => "PASSED",
            RunStatus::Failed => "FAILED",
        }
    }

    fn summary_line(&self) -> String {
        match self.status() {
            RunStatus::NoTestsRun => "no tests were run.".to_string(),
            RunStatus::Passed => format!(
                "passed, {} test(s) across {} file(s) completed in {} ms.",
                self.discovered_tests,
                self.selected_files,
                self.duration.as_millis()
            ),
            RunStatus::Failed => format!(
                "failed, {} record(s) across {} file(s); {} discovered test(s); fail fast {}; {} ms.",
                self.error_records.len(),
                self.failed_files,
                self.discovered_tests,
                if self.no_fail_fast {
                    "disabled"
                } else {
                    "enabled"
                },
                self.duration.as_millis()
            ),
        }
    }

    fn render_failures(&self, output: &mut String) {
        let mut current_file: Option<&str> = None;
        let mut index_in_file = 0;

        for record in &self.error_records {
            if current_file != Some(record.filename.as_str()) {
                if current_file.is_some() {
                    writeln!(output).unwrap();
                }
                current_file = Some(record.filename.as_str());
                index_in_file = 0;
                writeln!(output, "[{}]", record.filename).unwrap();
            }

            index_in_file += 1;
            writeln!(
                output,
                "  {}. query_id: {}",
                index_in_file,
                record.query_id.as_deref().unwrap_or("unknown")
            )
            .unwrap();

            for line in record.detail.lines() {
                writeln!(output, "     {line}").unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_report_for_no_tests_run() {
        let report = RunReport::new("MySQL", 0, 0, false, true, Duration::from_millis(7), vec![]);

        let rendered = report.render();

        assert!(rendered.contains("Status: NO TESTS RUN"));
        assert!(rendered.contains("Summary: no tests were run."));
    }

    #[test]
    fn render_report_for_success() {
        let report = RunReport::new("Http", 2, 5, true, true, Duration::from_millis(11), vec![]);

        let rendered = report.render();

        assert!(rendered.contains("Status: PASSED"));
        assert!(
            rendered.contains("Summary: passed, 5 test(s) across 2 file(s) completed in 11 ms.")
        );
        assert!(!rendered.contains("Failures:"));
    }

    #[test]
    fn render_report_groups_failures_by_file() {
        let report = RunReport::new(
            "MySQL",
            3,
            8,
            true,
            false,
            Duration::from_millis(19),
            vec![
                ErrorRecord {
                    filename: "b.test".to_string(),
                    query_id: None,
                    detail: "second error".to_string(),
                },
                ErrorRecord {
                    filename: "a.test".to_string(),
                    query_id: Some("query-1".to_string()),
                    detail: "first error\nwith more detail".to_string(),
                },
            ],
        );

        let rendered = report.render();

        assert!(rendered.contains("Status: FAILED"));
        assert!(rendered.contains("Failed files: 2"));
        assert!(
            rendered.contains(
                "[a.test]\n  1. query_id: query-1\n     first error\n     with more detail"
            )
        );
        assert!(rendered.contains("[b.test]\n  1. query_id: unknown\n     second error"));
        assert!(rendered.contains(
            "Summary: failed, 2 record(s) across 2 file(s); 8 discovered test(s); fail fast enabled; 19 ms."
        ));
    }
}
