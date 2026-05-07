---
type: moc
status: active | archived
scope: project | topic | area
hub: false
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [type/moc, project/<slug>, topic/<area>]
project: <slug>
related: []
---

# <Project name> — Project Map

> [!info] What this project is
> 두 문장. 첫째: 어떤 문제를 푸는가. 둘째: 현재 상태 — 활발한지,
> 일시 정지인지, 정리 중인지.

## Quick links

- **Repository**: [<org/repo>](https://github.com/org/repo)
- **Release notes**: [[Release Notes]] 폴더, 최신 [[v<latest>]]
- **Active sprint**: [[YYYY-Wnn Retrospective]] (또는 [[YYYY-Wnn Review]])
- **Onboarding**: 존재한다면 [[<Project> — Onboarding]]

## Current focus

- 최우선: [[note or task]]
- 보조: …

여기에 2~3개 항목만 둔다. 포커스가 바뀌면 이 섹션을 다시 쓴다 —
버전 히스토리는 git에 있다.

## Architecture Decisions

```dataview
TABLE number AS "ID", status, file.link AS "Note"
FROM #type/adr AND #project/<slug>
SORT number ASC
```

오픈 / 제안 상태의 ADR은 도드라진다 — 몇 주씩 `proposed`로 두지 말 것.
승인하거나, 거절하거나, 대체한다.

## Releases

```dataview
TABLE version, release_date, file.link AS "Notes"
FROM #type/release AND #project/<slug>
SORT release_date DESC
LIMIT 10
```

## Recent activity

지난 2주 동안 이 프로젝트와 닿은 노트들.

```dataview
LIST file.link
FROM #project/<slug>
WHERE file.cday >= date(today) - dur(14 days)
SORT file.cday DESC
LIMIT 20
```

## Open debug logs

```dataview
TABLE severity, created, file.link AS "Note"
FROM #type/debug AND #project/<slug>
WHERE status = "open"
SORT severity DESC, created DESC
```

## Known risks / watchlist

아직 버그는 아니지만 지켜볼 가치가 있는 것들. 각 불릿은 우려를 추적하는
노트로 링크한다.

- <Risk> — [[…]] 참고
- <Risk>

## Team / stakeholders

- <handle> — 역할
- <handle> — 역할

## Related MOCs

- [[MOC — <sibling project>]] — 공유 의존성 / 사용자
- [[MOC — <area>]] — 이 프로젝트가 속한 더 큰 영역

## Notes

위 섹션 어디에도 맞지 않지만 여기 도착한 사람이 알아야 할 것. 작게
유지하고, 한 주제가 커지면 별도 노트를 만들어 여기서 링크한다.
