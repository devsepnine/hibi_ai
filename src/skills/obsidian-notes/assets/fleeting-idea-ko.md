---
type: fleeting
status: new | processed | discarded
captured_at: YYYY-MM-DDTHH:MM
review_on: YYYY-MM-DD        # 기본값: captured_at으로부터 약 1주일 후
source: shower | walk | reading | conversation | meeting | dream | ...
created: YYYY-MM-DD
tags: [type/fleeting, project/<slug-or-none>, topic/<area>]
project: <slug or none>
related: []
---

# Thought — <one-line summary>

> [!tldr] The idea in one sentence
> 불꽃이 식기 전에 적는다. 나중에 다듬어도 좋지만, 시작을 미루지 말 것.
> 깔끔하지만 잊어버린 아이디어보다 어수선한 세 줄짜리 플리팅이 낫다.

## Raw capture

자유 형식. 불릿, 단편, 반쪽짜리 문장 — 뇌가 실제로 만들어낸 그대로.
아직 다듬지 말 것; 그건 리뷰 단계의 몫이다.

- <point>
- <point>
- <point>

## Why this might matter

이 생각이 어떤 문제나 맥락과 연결되는지 한두 문장. 다음 주에 이 노트를
다시 읽을 미래의 자신이 *왜 이걸 적어둘 가치가 있다고 생각했는지*
이해해야 한다.

## Links (maybe)

관련 노트에 대한 빠른 추측 — 이 단계에선 실제로 존재할 필요 없음.

- [[?? Adjacent idea]]
- [[?? Existing ADR this contradicts]]

확실하지 않은 링크는 `??`로 표시한다; 리뷰 단계에서 해결한다.

## Review

주간 리뷰 중에 (또는 `review_on`이 도래했을 때) 채운다.

**Decision**: promote | defer | discard

- **promote**일 때: 새 노트로 링크하고 `status: processed`로 변경,
  frontmatter에 `promoted_to: [[<new note>]]` 추가.
- **defer**일 때: `review_on`을 새 날짜로 미루고 이유를 기록.
- **discard**일 때: 한 줄 근거 — "당연한 것으로 판명",
  "이미 [[other note]]에 기록됨", "추구할 가치 없음".

Review on: YYYY-MM-DD

### Upgrade path cheatsheet

| 아이디어가 이런 종류라면… | 이렇게 승격 |
|-----------------|------------|
| 재사용 가능한 개념이나 패턴 | `learning` 노트 |
| 감사 가능성이 필요한 결정 | `adr` |
| 프로젝트별 TODO | 티켓 또는 `daily` 작업 |
| 관련 아이디어들의 모음 | `moc` |
| 책/기사 참조 | `book` 노트 (이 노트를 시드 링크로) |

플리팅 노트를 기본값으로 썩히지 말 것 — 더 나은 무언가가 되거나, 정직한
폐기를 받거나 둘 중 하나여야 한다. "아이디어 → X 이유로 폐기"의 감사
기록이 그냥 삭제하는 것보다 가치 있다.
