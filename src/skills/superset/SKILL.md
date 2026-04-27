---
name: superset
description: Apache Superset 대시보드, 차트, 데이터셋, SQL Lab 관리를 위한 MCP 도구 활용 패턴. Use whenever the user mentions Superset, dashboards, BI charts, 대시보드 생성/수정, Superset 차트, 데이터셋 메트릭, 계산된 컬럼, SQL Lab, BI 분석, Superset 필터, Apache Superset, 슈퍼셋. Also trigger when the user asks to "show charts on dashboard X", "add a metric to dataset Y", "create a Superset chart from this SQL", "find which dashboards use table Z", or any task involving Superset administration via MCP.
keywords: [superset, 슈퍼셋, dashboard, 대시보드, bi, chart, 차트, dataset, 데이터셋, sql-lab, apache-superset]
---

# Apache Superset via MCP

Apache Superset의 대시보드, 차트, 데이터셋, SQL Lab을 MCP 도구(`mcp__superset__*`)로 관리하는 실전 패턴.

## When to Use

MCP 도구가 연결된 환경에서 다음 작업을 할 때 바로 사용:
- 대시보드/차트/데이터셋 탐색 및 감사
- 차트 신규 생성, 기존 차트 파라미터/필터 변경
- 데이터셋 컬럼·메트릭·계산된 컬럼 추가/수정
- SQL Lab에서 직접 쿼리 실행 또는 SQL 일괄 치환
- 대시보드에 차트 붙이기/떼기, 설정 편집

**반드시 MCP로:** 정확한 ID·스키마가 필요한 작업. Superset 웹 UI 재현 또는 스크린샷 추측 금지.

## 필수 호출 순서 (Discovery → Read → Write)

Superset은 ID 기반 API이므로, 모든 변경 작업은 다음 3단계를 지켜야 안전합니다:

1. **Discovery** — `list_*` 로 대상 후보를 찾는다.
2. **Read** — `get_*` 로 현재 상태를 읽고 스키마를 검증한다.
3. **Write** — `create_*` / `update_*` / `set_*` 호출.

```
ex) 차트 필터 변경
  list_charts(search="user_funnel") → chart_id 확정
  get_chart_filters(chart_id) → 현재 필터 구조 파악
  set_chart_filters(chart_id, new_filters)
```

> 중간 단계를 건너뛰면 존재하지 않는 컬럼/메트릭으로 차트를 깨뜨리기 쉽다. **현재 상태를 먼저 읽어라.**

## Core Workflows

### 1. 대시보드 탐색

```
list_dashboards()                        # 전체 목록
get_dashboard_config(dashboard_id)       # 대시보드 메타 + 레이아웃
get_dashboard_charts(dashboard_id)       # 속한 차트 목록
get_dashboard_filters(dashboard_id)      # 네이티브 필터
get_dashboard_chart_query_context(...)   # 각 차트의 실제 쿼리 컨텍스트
```

**패턴: "이 대시보드는 어떤 테이블을 쓰는지 알려줘"**

```
1) get_dashboard_charts(id) → chart ids
2) 각 chart_id에 대해 get_chart_params → datasource_id 수집
3) 수집한 datasource_id 집합으로 get_dataset → table_name
```

### 2. 차트 생성 / 수정

**신규 차트 만들기 (표준 흐름):**

```
1) list_datasets(search="...")            # 대상 데이터셋 찾기
2) get_dataset_columns(dataset_id)        # 사용 가능한 컬럼 확인
3) get_dataset_metrics(dataset_id)        # 사용 가능한 메트릭 확인
4) create_chart({
     slice_name,
     viz_type,
     datasource_id,
     datasource_type: "table",
     params: { ... }                      # viz_type에 맞는 params
   })
5) (옵션) add_chart_to_dashboard(chart_id, dashboard_id)
```

**기존 차트 수정:**

```
get_current_chart_config(chart_id)   # 항상 먼저 현재 config를 읽고
→ 필요한 필드만 덮어쓰기
update_chart(chart_id, { params: {...merged_params} })
```

> `update_chart`는 머지가 아니라 덮어쓰기에 가깝다. 반드시 현재 config를 먼저 읽고 수정 필드만 바꿔서 전체를 다시 보내라.

### 3. 차트 필터 제어

```
get_chart_filters(chart_id)               # 현재 필터 dict 확인
set_chart_filters(chart_id, filters)      # 필터 덮어쓰기
```

필터 구조는 차트마다 `adhoc_filters` / `filters` / `time_range` 형태가 섞여 있으므로 반드시 **get으로 먼저 모양을 확인**한다.

### 4. 데이터셋 관리

```
list_datasets(search="...")
get_dataset(dataset_id)
get_dataset_columns(dataset_id)
get_dataset_metrics(dataset_id)
refresh_dataset_schema(dataset_id)        # 원천 테이블 변경 후 재동기화
```

**데이터셋 스키마가 바뀌었을 때:**

```
1) refresh_dataset_schema(dataset_id)
2) get_dataset_columns(dataset_id) 로 결과 확인
3) 필요하면 update_dataset(dataset_id, {...}) 로 표시 이름·타입 조정
```

### 5. 메트릭 & 계산된 컬럼

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

**삭제 전 영향도 점검:**

```
삭제하려는 metric/column을 list_charts에서 참조하는 차트가 있는지 먼저 확인.
execute_sql 로 superset 메타 DB 직접 조회 금지 — MCP 도구로 충분.
```

### 6. SQL Lab

```
execute_sql(database_id, sql, schema?)    # 조회/검증용 SELECT
find_and_replace_in_sql(find, replace)    # 저장된 쿼리 일괄 치환
```

`execute_sql` 은 **읽기 전용 검증**용으로만 쓴다. DDL/DML은 원천 DB 관리 파이프라인을 거쳐야 한다.

## 일관된 작업 템플릿

### Template A: 신규 차트를 대시보드에 붙이기

```
1. list_datasets → dataset_id
2. get_dataset_columns / get_dataset_metrics → 사용할 필드 확정
3. create_chart(params) → chart_id
4. get_chart_params(chart_id) → 제대로 저장됐는지 검증
5. list_dashboards → dashboard_id
6. add_chart_to_dashboard(chart_id, dashboard_id)
7. get_dashboard_charts(dashboard_id) → 포함됐는지 확인
```

### Template B: 차트 필터 업데이트 (안전판)

```
1. get_current_chart_config(chart_id)      # 현재 전체 config 스냅샷
2. get_chart_filters(chart_id)             # 현재 필터
3. 사용자가 원하는 변경사항 반영한 new_filters 계산
4. set_chart_filters(chart_id, new_filters)
5. get_chart_filters(chart_id) 로 반영 재확인
```

### Template C: 데이터셋에 메트릭 추가 + 차트에 반영

```
1. get_dataset_metrics(dataset_id)         # 중복 금지 — 이미 있는지 체크
2. create_dataset_metric(dataset_id, {...})
3. list_charts(datasource_id=..., search=...) # 메트릭이 들어갈 차트 탐색
4. 대상 차트마다:
     get_current_chart_config → params 수정 → update_chart
5. get_chart_params(chart_id) 로 메트릭이 실제로 사용 중인지 확인
```

## Viz Type별 params 최소 필드

`create_chart` / `update_chart` 시 params 에 **비즈 타입마다 필수 필드가 다르다**. 전부 외우지 말고, 동일 viz_type의 기존 차트에서 `get_current_chart_config`로 샘플을 떠서 구조를 모방하라.

대표적인 필수 키:

| viz_type         | 필수 params 키                                           |
|------------------|---------------------------------------------------------|
| table            | `groupby`, `metrics`, `row_limit`                        |
| big_number_total | `metric`                                                 |
| pie              | `groupby`, `metric`                                      |
| line / area      | `x_axis`, `metrics`, `groupby?`, `time_grain_sqla`       |
| bar              | `x_axis`, `metrics`, `groupby?`                          |

> **샘플 복제 전략**: 이미 프로덕션에서 잘 돌고 있는 동일 viz_type 차트의 config를 `get_current_chart_config`로 읽어서 datasource/컬럼만 바꿔 새 차트 params 로 쓰면 실패율이 급격히 내려간다.

## Dashboard Config 편집

```
get_dashboard_config(dashboard_id)
→ position_json / json_metadata / css 등 구조 이해
→ 필요한 필드만 덮어써서 update_dashboard_config
```

`position_json` 은 트리 구조이므로 손으로 만들지 말고 **기존 구조 위에서 최소 수정**한다. 새 차트 추가가 목적이라면 `add_chart_to_dashboard` 로 충분한 경우가 많다 — config 직접 편집은 레이아웃/탭 재구성 같은 고급 작업에만.

## 실수하기 쉬운 것들

1. **ID 직접 추측 금지**
   - 항상 `list_*` 로 얻은 ID만 사용. 이전 세션 ID를 기억해 재사용하지 말 것.

2. **update 시 params 전체 유실**
   - `update_chart(chart_id, { params: { metric: "x" } })` 로 호출하면 기존 params 가 날아간다.
   - 반드시 `get_current_chart_config` → spread → 변경 → 전체 params 송신.

3. **데이터셋 새로고침 없이 차트 생성**
   - 원천 테이블에 컬럼이 추가됐는데 `refresh_dataset_schema`를 안 하면 Superset은 그 컬럼을 모른다.

4. **필터 타입 혼동**
   - adhoc_filters (차트 내부) vs 네이티브 대시보드 필터 vs `time_range` 는 서로 다른 필드. get으로 모양 확인 후 수정.

5. **execute_sql 남용**
   - 분석용 쿼리만. 메타데이터 수정이나 DDL은 금지. 메트릭/컬럼 변경은 전용 MCP 도구로 한다.

6. **삭제 전 영향도 확인 생략**
   - `delete_dataset` / `delete_dataset_metric` / `delete_calculated_column` 는 참조 차트가 깨진다.
   - 호출 전에 `list_charts` 로 참조 여부를 확인하고 사용자에게 승인 받을 것.

## 검증 체크리스트

작업 완료 전에 항상 다음을 MCP 도구로 재확인:

- [ ] 대상 ID가 `list_*` 결과와 일치하는가
- [ ] 변경 전 상태를 `get_*` 로 스냅샷 잡았는가
- [ ] 변경 후 `get_*` 재호출로 반영 결과를 확인했는가
- [ ] 삭제성 작업이면 참조 차트/대시보드 영향도를 점검했는가
- [ ] 신규 차트라면 대시보드 연결(add_chart_to_dashboard)까지 끝냈는가

## Quick Reference (주요 MCP 도구)

| 영역        | 도구                                                                                     |
|-------------|------------------------------------------------------------------------------------------|
| Database    | `list_databases`                                                                         |
| Dashboard   | `list_dashboards`, `get_dashboard_config`, `get_dashboard_charts`, `get_dashboard_filters`, `get_dashboard_chart_query_context`, `update_dashboard_config`, `add_chart_to_dashboard`, `remove_chart_from_dashboard` |
| Chart       | `list_charts`, `create_chart`, `update_chart`, `get_current_chart_config`, `get_chart_params`, `get_chart_filters`, `set_chart_filters` |
| Dataset     | `list_datasets`, `create_dataset`, `update_dataset`, `delete_dataset`, `get_dataset`, `get_dataset_columns`, `get_dataset_metrics`, `refresh_dataset_schema` |
| Metric      | `create_dataset_metric`, `update_dataset_metric`, `delete_dataset_metric`                |
| Calc Column | `create_calculated_column`, `update_calculated_column`, `delete_calculated_column`       |
| SQL Lab     | `execute_sql`, `find_and_replace_in_sql`                                                 |

**Remember:** Superset 작업은 읽기→확인→쓰기. MCP 도구로 정확한 ID·스키마를 확보한 뒤에만 변경한다. 추측 금지, 샘플 복제 권장.
