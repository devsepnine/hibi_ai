---
type: adr
number: NNNN
status: proposed | accepted | superseded | deprecated
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [type/adr, project/<slug>, topic/<area>, stage/<rfc|implementation|rollout>]
project: <slug>
deciders: ["<handle>"]
supersedes: null        # 이 ADR이 이전 결정을 대체하는 경우 "[[ADR-0007 ...]]"
supersededBy: null      # 이 ADR이 폐기될 때 채워 넣음
related: []
aliases: ["<Alt title>"]
---

# ADR-NNNN: <간결한 결정 제목>

> [!info] Status — <status>
> <상태는 의사결정자가 승인하면 `proposed` → `accepted`로 전환된다. 이후 다른
> 결정으로 대체되면 `superseded`로 변경하고 `supersededBy`를 설정한다.
> 폐기된 ADR은 절대 삭제하지 말 것 — 감사 기록의 일부다.>

## Context

지금 결정이 필요한 이유는 무엇인가? 강제 요인(새로운 요구사항, 확장의 어려움,
규제 변경, 포스트모템 결과)을 기술한다. 이 섹션은 결정 자체가 아니라
**왜 결정해야 하는가**에 초점을 맞춘다.

인접 자료를 링크한다.
- 선행 작업: [[ADR-MMMM …]]
- 트리거가 된 사건 / 디버그 노트: [[Debug YYYY-MM-DD …]]

## Constraints

- 필수 조건 (협상 불가능한 강제 요구사항)
- 있으면 좋은 조건 (의사결정 시 타이브레이커)
- 범위 외 (여기서 명시적으로 결정하지 않는 항목)

## Options considered

### Option A — <name>

- 형태: 한 단락 요약.
- 장점: ...
- 단점: ...

### Option B — <name>

- 형태: ...
- 장점: ...
- 단점: ...

### Option C — <name>

- 형태: ...
- 장점: ...
- 단점: ...

## Decision

> [!success] Decided: <option>
> 선택을 한 문장으로 다시 진술한다. 그 다음 2~5문장으로 **왜** 이 안이
> 다른 안들을 이겼는지 제약 조건을 인용해 설명한다. 이 단락이 미래의
> 자신이 다시 읽게 될 부분이다.

## Consequences

### Positive

- 이 결정으로 무엇이 더 쉬워지고/빨라지고/안전해지는가.

### Negative

- 비용으로 받아들이기로 한 것은 무엇인가.

### Neutral / to watch

- 영향을 아직 가늠할 수 없는 부수 효과.

## Implementation notes

- 마일스톤 계획 (간단한 불릿 목록; 상세한 계획은 티켓으로 미룸)
- 롤아웃 / 롤백 전략
- 이 결정이 옳았는지 알려줄 지표

## Related

- [[<이 결정을 출시하는 릴리스 노트>]]
- [[ADR-XXXX …]] 이 결정이 가능하게 하거나 의존하는 결정들
