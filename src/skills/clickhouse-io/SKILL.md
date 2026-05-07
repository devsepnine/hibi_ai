---
name: clickhouse-io
description: ClickHouse database patterns, query optimization, analytics, and data engineering best practices for high-performance analytical workloads. Use when 클릭하우스 쿼리, ClickHouse 최적화, 분석 데이터 처리, 데이터 엔지니어링.
keywords: [clickhouse, 클릭하우스, analytics, 분석, data-engineering, 데이터엔지니어링]
---

# ClickHouse Analytics Patterns

Column-oriented OLAP DBMS optimized for fast analytical queries on large datasets. Strengths: compression, parallel/distributed execution, real-time aggregation.

Docs: https://clickhouse.com/docs

## Engine Selection

| Engine | Use Case | Key Trait |
|---|---|---|
| `MergeTree` | Default analytical tables | Partition + ORDER BY index |
| `ReplacingMergeTree` | Dedup by key (multi-source ingest) | Keeps latest by version/insert order |
| `AggregatingMergeTree` | Pre-aggregated metrics | Stores `*State` aggregate functions |
| `SummingMergeTree` | Numeric sums by key | Auto-sums on merge |

Common skeleton:

```sql
CREATE TABLE t (
    date Date, key String, value UInt64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, key)
SETTINGS index_granularity = 8192;
```

`AggregatingMergeTree` reads use `*Merge` to finalize: `sumMerge(volume)`, `uniqMerge(users)`, `countMerge(trades)`. Writes (typically via materialized view) emit `*State`: `sumState(amount)`, `uniqState(user_id)`.

Docs: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family

## Query Optimization

**Filter order matters.** Put indexed columns (PARTITION BY + ORDER BY prefix) first; avoid leading `LIKE '%...%'` on non-indexed columns.

```sql
-- Good: hits partition + sort key
SELECT * FROM t
WHERE date >= '2025-01-01' AND key = 'k1' AND value > 1000
ORDER BY date DESC LIMIT 100;
```

**Aggregations:** prefer ClickHouse-native functions.

| Goal | Function |
|---|---|
| Distinct count | `uniq(col)` (HLL approx) / `uniqExact` |
| Percentile | `quantile(0.95)(col)` |
| Conditional count | `countIf(cond)` / `sumIf(col, cond)` |
| Time bucket | `toStartOfHour/Day/Month(ts)` |

**Window functions** work with standard SQL syntax (`OVER (PARTITION BY ... ORDER BY ...)`).

Docs: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

## Materialized Views (Real-time Aggregation)

```sql
CREATE MATERIALIZED VIEW stats_hourly_mv TO stats_hourly AS
SELECT toStartOfHour(timestamp) AS hour, market_id,
       sumState(amount) AS total_volume,
       uniqState(user_id) AS unique_users
FROM trades GROUP BY hour, market_id;
```

Read with `*Merge` against the target table. MVs trigger on INSERT into the source — design source schema for write throughput, MV target for read patterns.

Docs: https://clickhouse.com/docs/en/sql-reference/statements/create/view#materialized-view

## Insertion Patterns

**Batch, do not loop single inserts.** ClickHouse merges parts in background; each INSERT creates a part, so one-row inserts cause merge pressure.

```typescript
// Batch insert (recommended)
await clickhouse.insert({
  table: 'trades',
  values: tradesBatch,        // array of rows
  format: 'JSONEachRow',
})

// Streaming for continuous ingest: use HTTP POST with chunked body
// or the async_insert engine setting for high-frequency producers.
```

Use parameterized queries (`{var:Type}`) to avoid string interpolation injection. For very high-frequency writes, enable `async_insert=1` server-side rather than batching on the client.

Docs: https://clickhouse.com/docs/en/optimize/asynchronous-inserts

## Performance Monitoring

```sql
-- Slow queries (last hour)
SELECT query_id, query_duration_ms, read_rows, memory_usage, query
FROM system.query_log
WHERE type = 'QueryFinish' AND event_time >= now() - INTERVAL 1 HOUR
  AND query_duration_ms > 1000
ORDER BY query_duration_ms DESC LIMIT 10;

-- Table sizes
SELECT database, table, formatReadableSize(sum(bytes)) size, sum(rows) rows
FROM system.parts WHERE active GROUP BY database, table ORDER BY sum(bytes) DESC;
```

Other useful system tables: `system.merges`, `system.mutations`, `system.parts_columns`, `system.processes`.

Docs: https://clickhouse.com/docs/en/operations/system-tables

## Common Analytics Recipes

```sql
-- Daily active users
SELECT toDate(timestamp) day, uniq(user_id) dau
FROM events WHERE timestamp >= today() - 30 GROUP BY day ORDER BY day;

-- Funnel (per session)
SELECT countIf(step='view') v, countIf(step='click') c, countIf(step='buy') b
FROM events WHERE event_date = today() GROUP BY session_id;

-- Cohort retention by signup month
SELECT toStartOfMonth(signup_date) cohort,
       dateDiff('month', cohort, toStartOfMonth(activity_date)) months,
       uniq(user_id) active
FROM user_activity GROUP BY cohort, months ORDER BY cohort, months;
```

## Pipeline Patterns

- **ETL**: extract from OLTP (Postgres/MySQL) on schedule, transform to analytical schema, batch insert.
- **CDC**: Postgres `LISTEN/NOTIFY` or Debezium-style stream → append-only event table in ClickHouse; use `ReplacingMergeTree` if you need current-state views.
- Prefer denormalized tables for analytics; avoid heavy JOINs across large tables. Use dictionaries (`dictGet`) for low-cardinality lookups.

Docs: https://clickhouse.com/docs/en/sql-reference/dictionaries

## Best Practices

| Area | Do | Avoid |
|---|---|---|
| Partition | By month/day, low count | Per-hour or per-user partitions |
| ORDER BY | Frequent filter columns first, consider cardinality | Random column order |
| Types | Smallest fit (`UInt32`), `LowCardinality(String)`, `Enum` | Always-`String`, `Nullable` when avoidable |
| Reads | Specify columns | `SELECT *`, `FINAL`, many JOINs |
| Writes | Batch / async_insert | Per-row INSERT loops |
| Ops | Track slow queries, merges, disk usage | Ignoring `system.query_log` |

ClickHouse rewards schema design tuned to query patterns: pick partitioning + ordering to make hot queries scan minimal granules, push aggregation to materialized views, and batch writes.
