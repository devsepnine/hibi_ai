# Frontmatter Conventions

이 skill에서 emit하는 모든 노트는 YAML frontmatter 블록으로 시작한다. 스키마는 안정적이어서 Obsidian dataview 쿼리, 그래프 필터, 폴더 인덱스 노트가 필드 이름에 의존할 수 있다.

## Canonical shape

```yaml
---
type: release | adr | retro | debug | learning | daily | weekly | meeting | moc | book | fleeting
status: <per-type, see below>
created: YYYY-MM-DD
updated: YYYY-MM-DD      # optional; add when editing an older note
tags: [type/<cat>, project/<name>, topic/<area>, ...]
project: <project slug>
related: ["[[Note A]]", "[[Note B]]"]
aliases: ["Alt Title"]   # optional
---
```

## Field reference

### `type` (required)
어떤 template이 적용되고 어떤 `status` 값이 합법인지 제어한다.

| Value | Template |
|-------|----------|
| `release`  | Release notes |
| `adr`      | Architecture Decision Record |
| `retro`    | Sprint/iteration retrospective |
| `debug`    | Debugging session log |
| `learning` | Library/tool/pattern learning note |
| `daily`    | Daily log / dev journal |
| `weekly`   | Weekly review |
| `meeting`  | Meeting minutes |
| `moc`      | Map of Content / project index |
| `book`     | Book / article capture |
| `fleeting` | 빠르게 캡처되어 나중에 리뷰되는 빠른 아이디어 |

여기서 새 type을 만들지 마라 — 먼저 새 template을 추가한다.

### `status` (required)
허용되는 값은 `type`에 의존한다.

| type | status values | Transitions |
|------|---------------|-------------|
| `release`  | `draft` → `published` | 버전이 잘릴 때까지 `draft` |
| `adr`      | `proposed` → `accepted` → `superseded` \| `deprecated` | superseded ADR을 절대 삭제하지 않는다 — 표시하고 frontmatter의 `supersededBy`에 link한다 |
| `retro`    | `active` → `closed` | 액션 아이템이 열려있는 동안 `active` |
| `debug`    | `open` → `resolved` → `archived` | 수정이 land되면 `resolved`로 뒤집기; 검색을 조용히 하기 위해 몇 달 후 `archived` 추가 |
| `learning` | `draft` → `stable` → `stale` | API가 콘텐츠를 무효화할 만큼 변경되면 `stale` |
| `daily`    | `active` → `closed` | 하루 끝나거나 다음 daily가 시작될 때 `closed` |
| `weekly`   | `active` → `closed` | 주중 채우는 동안 `active`; 리뷰 후 `closed` |
| `meeting`  | `scheduled` → `captured` → `actioned` \| `archived` | 모든 액션 아이템이 follow-up을 link하면 `actioned`; 90일 이상 후 `archived` |
| `moc`      | `active` → `archived` | 프로젝트/주제가 끝날 때 `archived`; 역사를 위해 보관 |
| `book`     | `reading` → `completed` → `revisited` | 재독 후 `revisited`; 아래에 새 revisit 로그 추가 |
| `fleeting` | `new` → `processed` \| `discarded` | 콘텐츠가 learning/ADR/MOC로 승급되면 `processed`; 결과가 없으면 `discarded` |

### `created` (required), `updated` (optional)
ISO `YYYY-MM-DD`. 노트에서 백데이팅한다면 "today"가 아닌 생성 날짜를 사용한다.

### `tags` (required)
세 축이 필요하다; 더 많아도 괜찮다. 다음 섹션의 분류에 충실한다.

### `project` (required)
소문자 slug, 다단어이면 kebab-case. `project/<slug>` 태그와 일치시킨다. 예: `project: hibi-ai`이면 태그 `project/hibi-ai`.

### `related` (optional)
컨텍스트를 공유하는 노트로의 wikilink 배열. 양방향 링크는 Obsidian의 슈퍼파워이다 — 큰 임베드 블록 하나보다 두 개의 가벼운 `related` 항목을 선호한다.

### `aliases` (optional)
검색을 위한 대체 제목. ADR에 유용 (예: aliases가 기저 주제를 포함: `"Switch DB to Postgres"`).

### Type-specific extras
일부 type은 추가 frontmatter를 가진다. 같은 블록에 유지한다.

#### `adr`
```yaml
number: 12                        # zero-padded in filename, plain here
supersedes: "[[ADR-0007 ...]]"    # if applicable
supersededBy: "[[ADR-0019 ...]]"  # when this one is retired
deciders: ["alice", "bob"]
```

#### `release`
```yaml
version: 1.9.4
release_date: 2026-04-21
changelog_link: https://github.com/org/repo/releases/tag/v1.9.4
```

#### `retro`
```yaml
sprint: 2026-W16
team: platform
```

#### `debug`
```yaml
severity: low | medium | high | critical
resolved_in: "[[ADR-0012 ...]]" | "<PR #>" | null
```

#### `learning`
```yaml
library: zustand
version: 5.0.12
```

#### `daily`
```yaml
date: 2026-04-22          # same as created, kept so dataview can WHERE date = ...
day_of_week: Tuesday      # optional; templater often injects this
week: 2026-W17            # ISO week; lets weekly reviews glob by prefix
```

#### `weekly`
```yaml
week: 2026-W17
week_start: 2026-04-20    # Monday of the week
week_end:   2026-04-26
```

#### `meeting`
```yaml
date: 2026-04-22
start_time: "14:00"
duration_min: 30
attendees: ["alice", "bob"]
meeting_type: sync | review | 1on1 | planning | retro | adhoc
agenda_link: "https://..."   # or "[[Agenda ...]]"
```

#### `moc`
```yaml
scope: project | topic | area    # what the MOC curates
hub: true                        # set on the single "front door" MOC for a vault section
```

#### `book`
```yaml
author: "John Ousterhout"
title: "A Philosophy of Software Design"
isbn: "9781732102200"
year: 2018
rating: 4                  # optional 1..5
finished_on: 2026-04-15
```

#### `fleeting`
```yaml
captured_at: 2026-04-22T14:37
review_on: 2026-04-29      # default ~1 week out; dataview surfaces when due
source: "shower" | "reading" | "walk" | "conversation" | ...
```

## Tag taxonomy

세 개의 필수 축, 각각 prefix되어 root tag tree가 정리된 상태로 유지된다:

### `type/<category>`
`type` field를 미러링한다.
- `type/release`, `type/adr`, `type/retro`, `type/debug`, `type/learning`
- `type/daily`, `type/weekly`, `type/meeting`
- `type/moc`, `type/book`, `type/fleeting`

### `project/<slug>`
`project` field와 같은 slug. 예:
- `project/hibi-ai`, `project/installer`, `project/dashboard-frontend`

### `topic/<area>`
도메인 영역 — 가능하면 기존 vault 컨벤션을 사용한다. 일반적인 것:
- `topic/auth`, `topic/perf`, `topic/build`, `topic/ci`, `topic/db`
- `topic/ui`, `topic/api`, `topic/devx`, `topic/ops`, `topic/security`
- `topic/testing`, `topic/docs`

새 topic 태그는 절약해서 추가한다. 한 노트만 사용하는 태그를 만드는 것보다 기존을 재사용하는 것을 선호한다.

### Optional axes

- `stage/<phase>` — `stage/rfc`, `stage/implementation`, `stage/rollout`
- `tech/<stack>` — `tech/rust`, `tech/typescript`, `tech/react`
- `owner/<person>` — vault가 태그로 소유권을 추적한다면

`status/...` 태그는 피한다 — `status`는 자체 frontmatter field를 가진다.

## Worked example

```yaml
---
type: adr
number: 12
status: accepted
created: 2026-04-21
updated: 2026-04-22
tags: [type/adr, project/hibi-ai, topic/build, tech/rust, stage/implementation]
project: hibi-ai
deciders: ["owenkim"]
supersedes: "[[ADR-0007 Local-only installer sync]]"
related: ["[[v1.9.4 Release Notes]]", "[[Sync bundled cache design]]"]
aliases: ["Homebrew sync fix", "find_git_root marker"]
---

# ADR-0012: Scope `find_git_root` to the hibi_ai repo

...
```

Dataview는 이제 이를 슬라이스할 수 있다: "2026의 project/hibi-ai에 대한 모든 accepted ADR", "topic/rust 태그가 있는 모든 학습 노트" 등 본문을 파싱하지 않고도.
