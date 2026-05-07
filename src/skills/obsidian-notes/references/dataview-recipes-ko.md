# Dataview Recipes

Dataview는 frontmatter + tag를 쿼리 가능한 데이터베이스로 바꾸는 커뮤니티 플러그인이다. 대부분의 MOC 노트, 주간 리뷰, 프로젝트 개요는 손으로 리스트를 유지하기보다 dataview 블록을 임베드하는 것에서 이득을 본다.

이 레시피들은 skill의 frontmatter 스키마를 가정한다 ([frontmatter-conventions.md](frontmatter-conventions.md) 참조).

## Query types

- `TABLE` — 선택된 컬럼이 있는 행
- `LIST` — 평면 글머리표 리스트
- `TASK` — vault 전반의 체크박스 (`- [ ]`)
- `CALENDAR` — date field로 heat-map (DV Calendar 플러그인 필요)

`dataview` (일반 DQL) 또는 `dataviewjs` (더 많은 제어를 위한 JavaScript)로 태그된 fenced 블록으로 모든 query를 wrap한다.

## Per-type recipes

### All accepted ADRs for a project, newest first

````markdown
```dataview
TABLE
  number AS "ID",
  status,
  file.link AS "Note",
  created
FROM #type/adr AND #project/hibi-ai
WHERE status = "accepted"
SORT created DESC
```
````

### Superseded / deprecated ADRs (audit trail)

````markdown
```dataview
TABLE
  file.link AS "ADR",
  status,
  supersededBy AS "Superseded by"
FROM #type/adr
WHERE status = "superseded" OR status = "deprecated"
SORT file.name ASC
```
````

### Open debug logs with severity

````markdown
```dataview
TABLE severity, created, file.link AS "Note"
FROM #type/debug
WHERE status = "open"
SORT severity DESC, created DESC
```
````

### Open action items across all retros

````markdown
```dataview
TASK
FROM #type/retro
WHERE !completed
GROUP BY file.link
```
````

미팅에도 같은 형태가 작동한다:

````markdown
```dataview
TASK
FROM #type/meeting
WHERE !completed
SORT file.cday DESC
```
````

### This week's daily notes

````markdown
```dataview
LIST
FROM #type/daily
WHERE week = "2026-W17"
SORT file.name ASC
```
````

동적 "current week" (dataviewjs):

````markdown
```dataviewjs
const now = new Date()
const year = now.getUTCFullYear()
// simple ISO week calc; for production use luxon via dv.luxon
const firstJan = new Date(Date.UTC(year, 0, 1))
const days = Math.floor((now - firstJan) / 86400000)
const week = Math.ceil((days + firstJan.getUTCDay() + 1) / 7)
const iso = `${year}-W${String(week).padStart(2, "0")}`

dv.list(
  dv.pages('#type/daily')
    .where(p => p.week === iso)
    .sort(p => p.file.name)
    .map(p => p.file.link)
)
```
````

### Weekly review auto-feed: what I shipped this week

````markdown
```dataview
LIST file.link
FROM (#type/release OR #type/adr)
WHERE file.cday >= date("2026-04-20") AND file.cday <= date("2026-04-26")
SORT file.cday ASC
```
````

`weekly-review` template에 이를 굽어 자동으로 채워지도록 한다.

### Fleeting notes due for review

````markdown
```dataview
TABLE captured_at, review_on, source
FROM #type/fleeting
WHERE status = "new" AND review_on <= date(today)
SORT review_on ASC
```
````

### Books in progress

````markdown
```dataview
TABLE author, rating, finished_on
FROM #type/book
WHERE status = "reading"
SORT file.cday DESC
```
````

### All meetings with a specific attendee

````markdown
```dataview
TABLE date, meeting_type, file.link AS "Note"
FROM #type/meeting
WHERE contains(attendees, "alice")
SORT date DESC
```
````

### MOC feed: latest notes added to a topic

````markdown
```dataview
LIST file.link
FROM #project/hibi-ai AND -#type/moc
WHERE file.cday >= date(today) - dur(14 days)
SORT file.cday DESC
LIMIT 20
```
````

## Common pitfalls

- **Tag vs field queries** — `FROM #type/adr`와 `WHERE type = "adr"`는 둘 다 작동하지만, tag 쿼리만 Obsidian의 캐시된 tag 인덱스를 사용할 수 있다. `FROM`에서 tag를 선호한다.
- **String fields as dates** — `created`가 문자열 `"2026-04-22"`라면, `date("2026-04-22")`와 비교한다 (사전식인 `<="..."`가 아니다).
- **Array contains** — `attendees: [alice, bob]`에 대해 `contains(array, value)`. 새 버전에서는 `"alice" in attendees`도 작동한다.
- **LIMIT before SORT is wrong** — DQL은 source 순서로 적용한다; `LIMIT 10 ... SORT ...`가 아닌 `SORT ... DESC LIMIT 10`을 작성한다.

## Performance

Dataview는 영향받는 범위에서 vault 변경 시마다 재쿼리한다. 눈에 띄게 느린 쿼리는 거의 항상 다음에서 온다:

- `FROM ""` (전체 vault) tag 필터 없이 — tag를 추가
- `file.content` regex 검색 — 피하고 frontmatter field를 사용
- 수백 개 노트에 걸친 unbounded `SORT` — `LIMIT`을 추가

## When to reach for `dataviewjs`

DQL이 형태를 표현할 수 없을 때만 JavaScript 모드를 사용한다:

- `GROUP BY`를 넘어선 집계 (커스텀 predicate에 의한 bucketing)
- 노트 간 수학 (totals, rolling averages)
- 동적 날짜 범위 ("this month", today에 상대적)
- Mutation (드물다 — Templater를 선호)

`dataview` DQL은 읽기 쉽고 플러그인 업데이트에서 더 잘 살아남는다. 정말 필요할 때만 JS에 손을 댄다.
