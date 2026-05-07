---
name: clickhouse-io
description: ClickHouse database patterns, query optimization, analytics, and data engineering best practices for high-performance analytical workloads. Use when 클릭하우스 쿼리, ClickHouse 최적화, 분석 데이터 처리, 데이터 엔지니어링.
keywords: [clickhouse, 클릭하우스, analytics, 분석, data-engineering, 데이터엔지니어링]
---

# ClickHouse 분석 패턴

대용량 데이터셋에서 빠른 분석 쿼리에 최적화된 컬럼 지향 OLAP DBMS. 강점: 압축, 병렬/분산 실행, 실시간 집계.

문서: https://clickhouse.com/docs

## 엔진 선택

| Engine | Use Case | Key Trait |
|---|---|---|
| `MergeTree` | 기본 분석 테이블 | Partition + ORDER BY 인덱스 |
| `ReplacingMergeTree` | 키별 중복 제거 (다중 소스 인입) | version/insert order 기준 최신 유지 |
| `AggregatingMergeTree` | 사전 집계 메트릭 | `*State` 집계 함수 저장 |
| `SummingMergeTree` | 키별 숫자 합계 | merge 시 자동 합산 |

공통 스켈레톤:

```sql
CREATE TABLE t (
    date Date, key String, value UInt64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, key)
SETTINGS index_granularity = 8192;
```

`AggregatingMergeTree` 읽기는 finalize를 위해 `*Merge`를 사용한다: `sumMerge(volume)`, `uniqMerge(users)`, `countMerge(trades)`. 쓰기 (보통 materialized view를 통해) 는 `*State`를 발행한다: `sumState(amount)`, `uniqState(user_id)`.

문서: https://clickhouse.com/docs/en/engines/table-engines/mergetree-family

## 쿼리 최적화

**필터 순서가 중요하다.** 인덱스된 컬럼 (PARTITION BY + ORDER BY 접두사) 을 먼저 두고, 비인덱스 컬럼에 선행 `LIKE '%...%'`는 피한다.

```sql
-- Good: hits partition + sort key
SELECT * FROM t
WHERE date >= '2025-01-01' AND key = 'k1' AND value > 1000
ORDER BY date DESC LIMIT 100;
```

**집계:** ClickHouse 네이티브 함수를 선호한다.

| Goal | Function |
|---|---|
| Distinct count | `uniq(col)` (HLL 근사) / `uniqExact` |
| Percentile | `quantile(0.95)(col)` |
| Conditional count | `countIf(cond)` / `sumIf(col, cond)` |
| Time bucket | `toStartOfHour/Day/Month(ts)` |

**윈도우 함수**는 표준 SQL 문법과 동작한다 (`OVER (PARTITION BY ... ORDER BY ...)`).

문서: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

## Materialized Views (실시간 집계)

```sql
CREATE MATERIALIZED VIEW stats_hourly_mv TO stats_hourly AS
SELECT toStartOfHour(timestamp) AS hour, market_id,
       sumState(amount) AS total_volume,
       uniqState(user_id) AS unique_users
FROM trades GROUP BY hour, market_id;
```

타깃 테이블에 대해 `*Merge`로 읽는다. MV는 소스로의 INSERT에서 트리거된다 — 소스 스키마는 쓰기 처리량 위주, MV 타깃은 읽기 패턴 위주로 설계한다.

문서: https://clickhouse.com/docs/en/sql-reference/statements/create/view#materialized-view

## 인입 패턴

**일괄 처리하라, 단일 INSERT를 반복하지 말 것.** ClickHouse는 백그라운드에서 part를 머지한다. 각 INSERT가 part를 만들기 때문에, 행 단위 INSERT는 머지 압박을 유발한다.

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

문자열 보간 인젝션 방지를 위해 매개변수화된 쿼리 (`{var:Type}`) 를 사용한다. 매우 고빈도 쓰기에는 클라이언트 배칭 대신 서버 측에서 `async_insert=1`을 활성화한다.

문서: https://clickhouse.com/docs/en/optimize/asynchronous-inserts

## 성능 모니터링

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

기타 유용한 시스템 테이블: `system.merges`, `system.mutations`, `system.parts_columns`, `system.processes`.

문서: https://clickhouse.com/docs/en/operations/system-tables

## 분석 레시피

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

## 파이프라인 패턴

- **ETL**: 스케줄에 맞춰 OLTP (Postgres/MySQL) 에서 추출, 분석 스키마로 변환, 일괄 INSERT.
- **CDC**: Postgres `LISTEN/NOTIFY` 또는 Debezium 스타일 스트림 → ClickHouse의 append-only 이벤트 테이블; 현재 상태 뷰가 필요하면 `ReplacingMergeTree` 사용.
- 분석에는 비정규화된 테이블을 선호한다; 대용량 테이블 간의 무거운 JOIN은 피한다. 저카디널리티 룩업에는 dictionary (`dictGet`) 를 사용한다.

문서: https://clickhouse.com/docs/en/sql-reference/dictionaries

## 모범 사례

| Area | Do | Avoid |
|---|---|---|
| Partition | 월별/일별, 적은 개수 | 시간별 또는 사용자별 파티션 |
| ORDER BY | 자주 필터링되는 컬럼 우선, 카디널리티 고려 | 임의의 컬럼 순서 |
| Types | 가장 작은 타입 (`UInt32`), `LowCardinality(String)`, `Enum` | 무조건 `String`, 회피 가능한 `Nullable` |
| Reads | 컬럼 명시 | `SELECT *`, `FINAL`, 다수의 JOIN |
| Writes | 일괄 / async_insert | 행 단위 INSERT 루프 |
| Ops | 느린 쿼리, 머지, 디스크 사용량 추적 | `system.query_log` 무시 |

ClickHouse는 쿼리 패턴에 맞춰 튜닝된 스키마 설계에 보상한다: 핫 쿼리가 최소 granule만 스캔하도록 partitioning + ordering을 선택하고, 집계는 materialized view로 밀어 넣고, 쓰기는 일괄 처리한다.
