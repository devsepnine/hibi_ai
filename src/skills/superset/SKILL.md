---
name: superset
description: Apache Superset 대시보드, 차트, 데이터셋, SQL Lab 관리를 위한 MCP 도구 활용 패턴. Use whenever the user mentions Superset, dashboards, BI charts, 대시보드 생성/수정, Superset 차트, 데이터셋 메트릭, 계산된 컬럼, SQL Lab, BI 분석, Superset 필터, Apache Superset, 슈퍼셋. Also trigger when the user asks to "show charts on dashboard X", "add a metric to dataset Y", "create a Superset chart from this SQL", "find which dashboards use table Z", or any task involving Superset administration via MCP.
keywords: [superset, 슈퍼셋, dashboard, 대시보드, bi, chart, 차트, dataset, 데이터셋, sql-lab, apache-superset]
---

# Apache Superset via MCP

Practical patterns for managing Apache Superset dashboards, charts, datasets, and SQL Lab through MCP tools (`mcp__superset__*`).

## When to Use

Use immediately when the MCP tools are connected and the task is one of:
- Exploring or auditing dashboards / charts / datasets
- Creating new charts or changing params/filters on existing charts
- Adding or modifying dataset columns, metrics, calculated columns
- Running queries directly in SQL Lab or doing bulk SQL find-and-replace
- Attaching/detaching charts to dashboards or editing dashboard config

**Always go through MCP** for anything that requires precise IDs or schemas. Do not reproduce the Superset web UI from memory or guess from screenshots.

## Required call order (Discovery → Read → Write)

Superset is an ID-based API, so every mutating operation must follow three steps:

1. **Discovery** — find candidate targets with `list_*`.
2. **Read** — fetch current state with `get_*` and validate the schema.
3. **Write** — call `create_*` / `update_*` / `set_*`.

```
ex) Change chart filters
  list_charts(search="user_funnel") → resolve chart_id
  get_chart_filters(chart_id) → understand current filter shape
  set_chart_filters(chart_id, new_filters)
```

> Skipping the middle step makes it easy to break a chart with non-existent columns/metrics. **Read current state first.**

## Core Workflows

### 1. Dashboard exploration

```
list_dashboards()                        # all dashboards
get_dashboard_config(dashboard_id)       # metadata + layout
get_dashboard_charts(dashboard_id)       # charts contained
get_dashboard_filters(dashboard_id)      # native filters
get_dashboard_chart_query_context(...)   # actual query context per chart
```

**Pattern: "tell me which tables this dashboard uses"**

```
1) get_dashboard_charts(id) → chart ids
2) for each chart_id call get_chart_params → collect datasource_id
3) for each datasource_id call get_dataset → table_name
```

### 2. Chart create / update

**Create a new chart (standard flow):**

```
1) list_datasets(search="...")            # find target dataset
2) get_dataset_columns(dataset_id)        # available columns
3) get_dataset_metrics(dataset_id)        # available metrics
4) create_chart({
     slice_name,
     viz_type,
     datasource_id,
     datasource_type: "table",
     params: { ... }                      # params required by viz_type
   })
5) (optional) add_chart_to_dashboard(chart_id, dashboard_id)
```

**Modify an existing chart:**

```
get_current_chart_config(chart_id)   # always read current config first
→ overwrite only the fields you need
update_chart(chart_id, { params: {...merged_params} })
```

> `update_chart` is closer to overwrite than merge. Always read the current config first, change only the target fields, and send the whole thing back.

### 3. Chart filter control

```
get_chart_filters(chart_id)               # current filter dict
set_chart_filters(chart_id, filters)      # overwrite filters
```

Filter shape varies by chart — `adhoc_filters` / `filters` / `time_range` are mixed — so always **inspect the current shape with `get`** before you mutate.

### 4. Dataset management

```
list_datasets(search="...")
get_dataset(dataset_id)
get_dataset_columns(dataset_id)
get_dataset_metrics(dataset_id)
refresh_dataset_schema(dataset_id)        # resync after upstream table changes
```

**When the dataset schema has changed:**

```
1) refresh_dataset_schema(dataset_id)
2) get_dataset_columns(dataset_id) to verify
3) if needed, update_dataset(dataset_id, {...}) to adjust display name / type
```

### 5. Metrics & calculated columns

```
create_dataset_metric(dataset_id, {
  metric_name: "total_gmv",
  expression: "SUM(gmv)",
  verbose_name: "Total GMV",
  d3format: ",.2f"
})

create_calculated_column(dataset_id, {
  column_name: "order_month",
  expression: "DATE_TRUNC('month', created_at)",
  type: "TIMESTAMP"
})
```

**Impact check before delete:**

```
Before deleting a metric/column, check whether any chart in list_charts references it.
Do NOT poke the Superset metadata DB directly via execute_sql — the MCP tools cover this.
```

### 6. SQL Lab

```
execute_sql(database_id, sql, schema?)    # SELECT-only validation
find_and_replace_in_sql(find, replace)    # bulk replace in saved queries
```

`execute_sql` is for **read-only validation** only. DDL/DML must go through the source DB management pipeline.

## Consistent task templates

### Template A: attach a new chart to a dashboard

```
1. list_datasets → dataset_id
2. get_dataset_columns / get_dataset_metrics → finalize fields
3. create_chart(params) → chart_id
4. get_chart_params(chart_id) → verify it saved correctly
5. list_dashboards → dashboard_id
6. add_chart_to_dashboard(chart_id, dashboard_id)
7. get_dashboard_charts(dashboard_id) → confirm inclusion
```

### Template B: update chart filters (safe variant)

```
1. get_current_chart_config(chart_id)      # snapshot full config
2. get_chart_filters(chart_id)             # current filters
3. compute new_filters from the user's request
4. set_chart_filters(chart_id, new_filters)
5. get_chart_filters(chart_id) to verify
```

### Template C: add a metric to a dataset and apply it to charts

```
1. get_dataset_metrics(dataset_id)         # avoid duplicates
2. create_dataset_metric(dataset_id, {...})
3. list_charts(datasource_id=..., search=...) # find target charts
4. for each target chart:
     get_current_chart_config → edit params → update_chart
5. get_chart_params(chart_id) to verify the metric is in use
```

## Minimum params per viz_type

`create_chart` / `update_chart` requires **different fields per viz_type**. Don't memorize them all — clone the structure from a working chart of the same `viz_type` via `get_current_chart_config`.

Common required keys:

| viz_type         | Required params keys                                     |
|------------------|----------------------------------------------------------|
| table            | `groupby`, `metrics`, `row_limit`                        |
| big_number_total | `metric`                                                 |
| pie              | `groupby`, `metric`                                      |
| line / area      | `x_axis`, `metrics`, `groupby?`, `time_grain_sqla`       |
| bar              | `x_axis`, `metrics`, `groupby?`                          |

> **Sample-clone strategy**: read the config of a known-good chart with the same viz_type via `get_current_chart_config`, swap datasource/columns, and reuse it as the new chart's params. Failure rate drops sharply.

## Dashboard config edits

```
get_dashboard_config(dashboard_id)
→ understand position_json / json_metadata / css
→ overwrite only required fields via update_dashboard_config
```

`position_json` is a tree — don't hand-craft it; **make minimal edits on top of the existing structure**. If your goal is just to add a chart, `add_chart_to_dashboard` is usually enough — direct config edits are for advanced layout/tab restructuring.

## Common mistakes

1. **Don't guess IDs**
   - Use only IDs returned from `list_*`. Don't reuse IDs from a previous session.

2. **Update wiping out params**
   - `update_chart(chart_id, { params: { metric: "x" } })` will drop the rest of params.
   - Always do `get_current_chart_config` → spread → modify → send the full params.

3. **Creating a chart without refreshing the dataset**
   - If the upstream table added a column and `refresh_dataset_schema` wasn't called, Superset doesn't know about it.

4. **Filter type confusion**
   - adhoc_filters (chart-internal) vs native dashboard filter vs `time_range` are different fields. Inspect via `get` first.

5. **Abusing execute_sql**
   - Analysis queries only. No metadata edits or DDL. Metric/column changes go through the dedicated MCP tools.

6. **Skipping impact analysis before delete**
   - `delete_dataset` / `delete_dataset_metric` / `delete_calculated_column` will break referencing charts.
   - Check references with `list_charts` before calling and get user approval.

## Verification checklist

Always re-verify with MCP tools before declaring done:

- [ ] Target ID matches the `list_*` result
- [ ] Pre-change state captured via `get_*`
- [ ] Post-change state confirmed by re-calling `get_*`
- [ ] For deletes, referencing charts/dashboards were checked
- [ ] For new charts, dashboard linkage (`add_chart_to_dashboard`) is complete

## Quick Reference (key MCP tools)

| Area        | Tools                                                                                     |
|-------------|------------------------------------------------------------------------------------------|
| Database    | `list_databases`                                                                         |
| Dashboard   | `list_dashboards`, `get_dashboard_config`, `get_dashboard_charts`, `get_dashboard_filters`, `get_dashboard_chart_query_context`, `update_dashboard_config`, `add_chart_to_dashboard`, `remove_chart_from_dashboard` |
| Chart       | `list_charts`, `create_chart`, `update_chart`, `get_current_chart_config`, `get_chart_params`, `get_chart_filters`, `set_chart_filters` |
| Dataset     | `list_datasets`, `create_dataset`, `update_dataset`, `delete_dataset`, `get_dataset`, `get_dataset_columns`, `get_dataset_metrics`, `refresh_dataset_schema` |
| Metric      | `create_dataset_metric`, `update_dataset_metric`, `delete_dataset_metric`                |
| Calc Column | `create_calculated_column`, `update_calculated_column`, `delete_calculated_column`       |
| SQL Lab     | `execute_sql`, `find_and_replace_in_sql`                                                 |

**Remember:** Superset work is read → verify → write. Always nail down accurate IDs/schemas via MCP before mutating. No guessing, prefer sample cloning.
