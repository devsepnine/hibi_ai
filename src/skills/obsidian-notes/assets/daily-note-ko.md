---
type: daily
status: active | closed
date: YYYY-MM-DD
day_of_week: Monday | Tuesday | ...
week: YYYY-Wnn
created: YYYY-MM-DD
tags: [type/daily, project/<slug-or-none>]
project: <slug or none>
related: []
---

# YYYY-MM-DD <Day name>

> [!info] Focus for today
> 한 문장 의도: 이것 하나만 끝내면 오늘이 좋은 하루가 되는 그 일.
> 떠오르지 않으면, 그 일을 정하는 것부터 한다.

## Plan

- [ ] <task 1> — 해당되면 티켓/노트로 링크
- [ ] <task 2>
- [ ] <task 3>

짧게 유지한다. 5개를 넘으면 하루치가 아니라 일주일치다.

## Notes during the day

타임스탬프가 붙은 진행 로그. 일이 일어나는 대로 항목을 추가한다 —
생각을 소리 내어 적는 메모, 관찰, 작은 결정들.

- `HH:MM` — <note>
- `HH:MM` — <note>

긴 생각은 별도 노트로 빼고 링크한다.
- `HH:MM` — 접근 X를 스파이크함 ([[Spike — approach X]] 참고)

## Meetings

오늘의 회의 노트들을 wikilink 목록으로 — 나중에 훑어보기에 좋다.

- [[YYYY-MM-DD <meeting slug>]]

## What I did

하루를 마치며 적는 불릿. 과거형. 구체적으로.

- 출시함 …
- 리뷰함 …
- 디버깅함 … ([[Debug YYYY-MM-DD <slug>]] 참고)

## What I learned

오늘의 깨달음을 기록한다. 따로 떼어 둘 만큼 큰 것은 학습/플리팅 노트로
링크한다.

- [[Fleeting YYYY-MM-DD-HHmm idea]] — 엣지에서의 캐싱 경계
- Zustand v5는 기본적으로 `Object.is`를 사용함을 확인 (→ [[Zustand — useShallow and v5 selector equality]])

## Tomorrow

내일의 자신이 차갑게 시작하지 않도록 다음 데일리의 씨앗을 심는다.

- [ ] … 후속 조치
- [ ] … ADR 초안 작성

## Related

- [[YYYY-Wnn]] — 이번 주 리뷰 (금요일에 채워짐)
- [[{{previous daily}}]]
