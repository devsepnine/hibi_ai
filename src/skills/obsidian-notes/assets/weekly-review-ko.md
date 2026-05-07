---
type: weekly
status: active | closed
week: YYYY-Wnn
week_start: YYYY-MM-DD
week_end:   YYYY-MM-DD
created: YYYY-MM-DD
tags: [type/weekly, project/<slug-or-none>]
project: <slug or none>
related: []
---

# Week YYYY-Wnn Review

> [!info] Week theme
> 한 주의 테마가 있었다면 명명한다. 없으면 "no theme"이라 적고 억지로
> 만들지 말 것 — 주간 리뷰의 노이즈는 몇 달에 걸쳐 누적된다.

## This week's dailies

리뷰 노트가 항상 최신 상태를 유지하도록 라이브 목록을 임베드한다.

```dataview
LIST file.link
FROM #type/daily
WHERE week = "YYYY-Wnn"
SORT file.name ASC
```

## Shipped / completed

- <thing> — [[release note]] 또는 티켓 링크
- <thing>

릴리스 / ADR frontmatter에서 자동 피드 (날짜 교체).

```dataview
LIST file.link
FROM (#type/release OR #type/adr)
WHERE file.cday >= date("YYYY-MM-DD") AND file.cday <= date("YYYY-MM-DD")
SORT file.cday ASC
```

## Learned

이번 주 학습을 긴 단락이 아니라 원자적 노트로 큐레이션한다.

- [[Zustand — useShallow and v5 selector equality]]
- <한 줄 시사점> — 이를 포착한 daily/fleeting 링크

## Metrics (optional)

의미 있는 1~3개를 추적한다. 예시.

| Metric | This week | Last week | Delta |
|--------|-----------|-----------|-------|
| Commits | 32 | 28 | +4 |
| Meetings (hrs) | 6 | 9 | −3 |
| Deep-work blocks | 7 | 5 | +2 |

행동을 바꾸지 않을 숫자는 추적하지 말 것. 어떤 숫자가 행동을 바꾸지
않으면 템플릿에서 제거한다.

## Fleeting notes review

```dataview
TABLE captured_at, source, review_on
FROM #type/fleeting
WHERE status = "new" AND review_on <= date(today)
SORT review_on ASC
```

마감된 각 플리팅에 대해: **promote** (새 노트), **defer** (`review_on`
연기), 또는 **discard** (표시하고 넘어감). 패턴은
[[vault-organization — Fleeting → Evergreen]] 참고.

## Open action items

회의와 회고에서.

```dataview
TASK
FROM #type/meeting OR #type/retro
WHERE !completed
GROUP BY file.link
```

## What went well

- <thing> — 통한 이유; 계속할 가치

## What didn't

- <thing> — 진짜 원인은 무엇인가 ("시간이 없었다"는 답이 아님)

구체적으로. "회의가 안 좋았다"는 노이즈; "수요일 오후 연속 세 개의
싱크가 릴리스를 막았다"는 실행 가능하다.

## Next week

- [ ] 최우선 목표:
- [ ] 두 번째 목표:
- [ ] 후속:

목표는 ~3개 이내. 금요일이면 완료 정의가 명확히 보이도록 작성한다.

## Related

- [[YYYY-W(nn-1)]] — 이전 주
- [[MOC — <active project>]]
