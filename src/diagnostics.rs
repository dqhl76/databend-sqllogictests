use sqllogictest::Runner;

use crate::report::NonDefaultSetting;
use crate::util::ColumnType;

pub(crate) struct FailureDiagnostics {
    pub(crate) query_id: Option<String>,
    pub(crate) non_default_settings: Vec<NonDefaultSetting>,
}

pub(crate) async fn capture_failure_diagnostics<D, M>(
    runner: &mut Runner<D, M>,
) -> FailureDiagnostics
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    let query_id = fetch_last_query_id(runner).await;
    let non_default_settings = fetch_non_default_settings(runner).await;

    FailureDiagnostics {
        query_id,
        non_default_settings,
    }
}

async fn fetch_last_query_id<D, M>(runner: &mut Runner<D, M>) -> Option<String>
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    fetch_query_rows(runner, "T", "SELECT LAST_QUERY_ID()")
        .await
        .and_then(|rows| rows.into_iter().next())
        .and_then(|row| row.into_iter().next())
}

async fn fetch_non_default_settings<D, M>(runner: &mut Runner<D, M>) -> Vec<NonDefaultSetting>
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    fetch_query_rows(
        runner,
        "TTTT",
        "SELECT name, value, default, level \
         FROM system.settings \
         WHERE value <> default \
         ORDER BY name",
    )
    .await
    .unwrap_or_default()
    .into_iter()
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

async fn fetch_query_rows<D, M>(
    runner: &mut Runner<D, M>,
    column_types: &str,
    sql: &str,
) -> Option<Vec<Vec<String>>>
where
    D: sqllogictest::AsyncDB<ColumnType = ColumnType>,
    M: sqllogictest::MakeConnection<Conn = D>,
{
    let script = format!("query {column_types}\n{sql}\n----\n");
    let records = sqllogictest::parse::<ColumnType>(&script).ok()?;
    let record = records.into_iter().next()?;
    if let sqllogictest::RecordOutput::Query { rows, .. } = runner.apply_record(record).await {
        Some(rows)
    } else {
        None
    }
}
