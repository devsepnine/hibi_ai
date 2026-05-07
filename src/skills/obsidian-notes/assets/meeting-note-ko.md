---
type: meeting
status: scheduled | captured | actioned | archived
date: YYYY-MM-DD
start_time: "HH:MM"
duration_min: 30
meeting_type: sync | review | 1on1 | planning | retro | adhoc
attendees: ["<handle>", "<handle>"]
agenda_link: null          # "[[Agenda ...]]" 또는 "https://..."
created: YYYY-MM-DD
tags: [type/meeting, project/<slug>]
project: <slug>
related: []
---

# YYYY-MM-DD <Meeting slug / topic>

> [!info] Purpose
> 한 문장: 이 회의가 해결하려는 것. 명명할 수 없다면 이메일/비동기
> 스레드로 대체할 수 있는지 검토한다.

## Attendees

- <handle> — 자명하지 않으면 역할
- <handle>

## Agenda

1. …
2. …
3. …

`agenda_link`에서 어젠다가 왔다면 임베드만 해도 된다.
`![[Agenda YYYY-MM-DD ...]]`

## Discussion

어젠다 항목별로 구조화. 각 항목에 미니 섹션을 두면 나중에 보는 사람이
전체 회의록을 다시 읽지 않고 점프할 수 있다.

### 1. <Topic>

- 제기된 핵심
- 공유된 데이터 ([[supporting note]] 링크 또는 숫자 붙여넣기)
- 의견 차이와 해결 (또는 미해결로 남았다는 사실)

### 2. <Topic>

…

## Decisions

> [!success] Decided
> 가능하면 담당자와 함께 번호 매긴 결정. ADR이 필요할 만큼 무거운
> 결정이라면 표시하고 작성한다:
> [[ADR-NNNN …]] — <owner>가 초안 작성 예정.

1. **<Decision>** — 한 줄 맥락.
2. **<Decision>** — …

아무것도 결정되지 않았다면 명시한다: `> [!question] Deferred —
no decision.` 그래야 독자가 놓친 게 있는지 의심하지 않는다.

## Action items

각 작업은 Obsidian 작업(`- [ ]`)이 되며, 담당자/마감일/후속 노트(있다면)
링크가 함께 붙는다.

- [ ] <Action> — owner: <handle> — due: YYYY-MM-DD — tracking:
  [[Follow-up note]] 또는 `<ticket url>`
- [ ] <Action> — owner: <handle> — due: YYYY-MM-DD

Dataview가 회의들에서 이들을 모아 주간 리뷰로 롤업한다.

## Open questions

- 아직 모르는 것. 이들이 다음 회의 어젠다의 씨앗이 되곤 한다.

## Parking lot

어젠다에 없었지만 제기된, 시간 안에 다루지 못한 주제. 다음 회의나
적절한 티켓으로 승격한다.

## Related

- [[Previous meeting in this series]]
- [[MOC — <project>]]
- 인접 결정: [[ADR-NNNN …]]
