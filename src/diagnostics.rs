use async_trait::async_trait;
use sqllogictest::Runner;

use crate::report::NonDefaultSetting;
use crate::util::ColumnType;

const LAST_QUERY_ID_COLUMN_TYPES: &str = "T";
const LAST_QUERY_ID_SQL: &str = "SELECT LAST_QUERY_ID()";
const NON_DEFAULT_SETTINGS_COLUMN_TYPES: &str = "TTTT";
const NON_DEFAULT_SETTINGS_SQL: &str = "SELECT name, value, default, level \
     FROM system.settings \
     WHERE value <> default \
     ORDER BY name";

pub(crate) struct FailureDiagnostics {
    pub(crate) query_id: Option<String>,
    pub(crate) non_default_settings: Vec<NonDefaultSetting>,
}

#[async_trait(?Send)]
trait DiagnosticsQueryExecutor {
    async fn query_rows(&mut self, column_types: &str, sql: &str) -> Option<Vec<Vec<String>>>;
}

struct RunnerDiagnosticsExecutor<'a, D, M>
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    runner: &'a mut Runner<D, M>,
}

#[async_trait(?Send)]
impl<D, M> DiagnosticsQueryExecutor for RunnerDiagnosticsExecutor<'_, D, M>
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    async fn query_rows(&mut self, column_types: &str, sql: &str) -> Option<Vec<Vec<String>>> {
        let script = format!("query {column_types}\n{sql}\n----\n");
        let records = sqllogictest::parse::<ColumnType>(&script).ok()?;
        let record = records.into_iter().next()?;
        if let sqllogictest::RecordOutput::Query { rows, .. } =
            self.runner.apply_record(record).await
        {
            Some(rows)
        } else {
            None
        }
    }
}

pub(crate) async fn capture_failure_diagnostics<D, M>(
    runner: &mut Runner<D, M>,
) -> FailureDiagnostics
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    let mut executor = RunnerDiagnosticsExecutor { runner };
    capture_failure_diagnostics_with_executor(&mut executor).await
}

async fn capture_failure_diagnostics_with_executor(
    executor: &mut impl DiagnosticsQueryExecutor,
) -> FailureDiagnostics {
    let query_id = executor
        .query_rows(LAST_QUERY_ID_COLUMN_TYPES, LAST_QUERY_ID_SQL)
        .await
        .and_then(extract_last_query_id);
    let non_default_settings = executor
        .query_rows(NON_DEFAULT_SETTINGS_COLUMN_TYPES, NON_DEFAULT_SETTINGS_SQL)
        .await
        .map(extract_non_default_settings)
        .unwrap_or_default();

    FailureDiagnostics {
        query_id,
        non_default_settings,
    }
}

fn extract_last_query_id(rows: Vec<Vec<String>>) -> Option<String> {
    rows.into_iter()
        .next()
        .and_then(|row| row.into_iter().next())
}

fn extract_non_default_settings(rows: Vec<Vec<String>>) -> Vec<NonDefaultSetting> {
    rows.into_iter()
        .filter_map(|row| match row.as_slice() {
            [name, value, default_value, level] => Some(NonDefaultSetting::new(
                name.clone(),
                value.clone(),
                default_value.clone(),
                level.clone(),
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct MockQueryResponse {
        column_types: &'static str,
        sql: &'static str,
        rows: Option<Vec<Vec<String>>>,
    }

    struct MockExecutor {
        responses: VecDeque<MockQueryResponse>,
    }

    #[async_trait(?Send)]
    impl DiagnosticsQueryExecutor for MockExecutor {
        async fn query_rows(&mut self, column_types: &str, sql: &str) -> Option<Vec<Vec<String>>> {
            let response = self.responses.pop_front().expect("missing mock response");
            assert_eq!(response.column_types, column_types);
            assert_eq!(response.sql, sql);
            response.rows
        }
    }

    #[test]
    fn extract_last_query_id_uses_first_cell() {
        let query_id = extract_last_query_id(vec![
            vec!["query-1".to_string(), "ignored".to_string()],
            vec!["query-2".to_string()],
        ]);

        assert_eq!(query_id, Some("query-1".to_string()));
    }

    #[test]
    fn extract_non_default_settings_ignores_malformed_rows() {
        let settings = extract_non_default_settings(vec![
            vec![
                "max_threads".to_string(),
                "8".to_string(),
                "16".to_string(),
                "SESSION".to_string(),
            ],
            vec!["broken".to_string()],
            vec![
                "timezone".to_string(),
                "UTC".to_string(),
                "SYSTEM".to_string(),
                "SESSION".to_string(),
            ],
        ]);

        assert_eq!(
            settings,
            vec![
                NonDefaultSetting::new("max_threads", "8", "16", "SESSION"),
                NonDefaultSetting::new("timezone", "UTC", "SYSTEM", "SESSION"),
            ]
        );
    }

    #[tokio::test]
    async fn capture_failure_diagnostics_combines_query_id_and_settings() {
        let mut executor = MockExecutor {
            responses: VecDeque::from([
                MockQueryResponse {
                    column_types: LAST_QUERY_ID_COLUMN_TYPES,
                    sql: LAST_QUERY_ID_SQL,
                    rows: Some(vec![vec!["query-1".to_string()]]),
                },
                MockQueryResponse {
                    column_types: NON_DEFAULT_SETTINGS_COLUMN_TYPES,
                    sql: NON_DEFAULT_SETTINGS_SQL,
                    rows: Some(vec![
                        vec![
                            "max_threads".to_string(),
                            "8".to_string(),
                            "16".to_string(),
                            "SESSION".to_string(),
                        ],
                        vec![
                            "timezone".to_string(),
                            "UTC".to_string(),
                            "SYSTEM".to_string(),
                            "SESSION".to_string(),
                        ],
                    ]),
                },
            ]),
        };

        let diagnostics = capture_failure_diagnostics_with_executor(&mut executor).await;

        assert_eq!(diagnostics.query_id, Some("query-1".to_string()));
        assert_eq!(
            diagnostics.non_default_settings,
            vec![
                NonDefaultSetting::new("max_threads", "8", "16", "SESSION"),
                NonDefaultSetting::new("timezone", "UTC", "SYSTEM", "SESSION"),
            ]
        );
        assert!(executor.responses.is_empty());
    }

    #[tokio::test]
    async fn capture_failure_diagnostics_defaults_when_queries_return_nothing() {
        let mut executor = MockExecutor {
            responses: VecDeque::from([
                MockQueryResponse {
                    column_types: LAST_QUERY_ID_COLUMN_TYPES,
                    sql: LAST_QUERY_ID_SQL,
                    rows: None,
                },
                MockQueryResponse {
                    column_types: NON_DEFAULT_SETTINGS_COLUMN_TYPES,
                    sql: NON_DEFAULT_SETTINGS_SQL,
                    rows: None,
                },
            ]),
        };

        let diagnostics = capture_failure_diagnostics_with_executor(&mut executor).await;

        assert_eq!(diagnostics.query_id, None);
        assert!(diagnostics.non_default_settings.is_empty());
        assert!(executor.responses.is_empty());
    }
}
