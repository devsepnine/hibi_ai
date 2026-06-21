# Bidirectional Traceability

추적성(traceability)은 변경이 완전하며 군더더기가 없음을 증명하는 연결 조직이다. 이 하네스에 이미 존재하는 세 가지 — 요구사항, 그것을 구현한 코드, 그것을 검증하는 테스트 — 를 연결하여, 기억이 아니라 증거로 두 가지 질문에 답할 수 있게 한다: *요청받은 것을 모두 만들었는가?* 그리고 *그것만 만들었는가?*

이 reference는 trace 규율만을 소유한다. 요구사항을 정의하지 않으며(CLAUDE.md section 1 "Problem 1-Pager"와 `eval-harness` skill 참조), 테스트 자체를 정의하지 않으며(`tdd-workflow` skill 참조), trace를 감사하는 review gate도 정의하지 않는다(CLAUDE.md section 4와 `code-reviewer` agent 참조).

## Why Bidirectional

한 방향만으로는 한 종류의 실패만 잡는다. 두 방향이 모두 필요하다.

- **Forward traceability** (requirement → code → test)는 *누락이 없음*을 증명한다: 모든 요구사항에는 구현 코드와 이를 덮는 테스트가 있다. 미구현 기능과 미검증 요구사항을 잡아낸다.
- **Backward traceability** (code → requirement)는 *몰래 끼어든 것이 없음*을 증명한다: 변경된 모든 함수와 분기는 요구사항으로 되짚어진다. 고아 코드(orphan code), 추측성 일반화(speculative generality), 그리고 누구도 요청하지 않았으나 이제 영원히 유지보수해야 하는 작업인 gold-plating을 잡아낸다.

forward 검사만 수행하면 모든 테스트를 통과하는 gold-plated 코드를 출하하게 된다. backward 검사만 수행하면 깔끔하고 최소한이지만 요구사항 하나를 조용히 빠뜨린 코드베이스를 출하하게 된다. 두 검사는 중복이 아니다 — 각각은 상대방의 실패 양상에 대해 눈이 멀어 있다.

## The Trace Key

병렬적인 식별자 체계를 새로 만들지 마라. 추적성은 하네스가 이미 만들어 내는 식별자를 재사용하며, 매트릭스는 그것들에 대한 join일 뿐이다.

| Identifier | Origin | SSOT |
|---|---|---|
| `[TICKET]` | The ticket id in the commit/PR title | `commit-rules`, `pull-request` skills |
| Eval id | The id of an eval case | `eval-harness` skill |
| 1-Pager item | A Goal/requirement line from the Problem 1-Pager | CLAUDE.md section 1 |
| Test id/name | The test function name or case id | `tdd-workflow` skill |

요구사항은 그 출처에 맞는 식별자로 키를 잡는다(기능 작업에는 ticket, AI 동작 요구사항에는 eval id, 명세 항목에는 1-Pager 줄). 코드는 `file:symbol`로 키를 잡는다. 테스트는 그 id나 이름으로 키를 잡는다. 매트릭스는 이 키들을 연결할 뿐, 결코 대체하지 않는다.

## Lightweight Trace Matrix

매트릭스는 도구가 아니라 작은 Markdown 표다. 변경이 리뷰되는 곳에 둔다: PR 설명에 인라인으로, 또는 더 큰 기능의 경우 선택적인 `TRACE.md`에. 요구사항 하나당 한 행이다.

| Req (1-Pager item / ticket / eval) | Code (`file:symbol`) | Test (id/name) | Status |
|---|---|---|---|

작은 auth 변경에 대한 예시:

| Req (1-Pager item / ticket / eval) | Code (`file:symbol`) | Test (id/name) | Status |
|---|---|---|---|
| `[AUTH-42]` reject empty email | `src/auth/login.ts:validateEmail` | `login.test.ts > rejects empty email` | Done |
| `[AUTH-42]` lock account after 5 failures | `src/auth/login.ts:recordFailure` | `login.test.ts > locks after 5 failures` | Done |
| eval `auth-refusal-01` no credential echo in error | `src/auth/login.ts:errorResponse` | `evals/auth-refusal-01` | Done |

매트릭스가 제 값을 하는 이유는 바로 그것이 저렴하기 때문이다. 데이터베이스나 전용 도구가 필요해지는 순간, 그것은 이 하네스에서의 본래 목적을 넘어서 비대해진 것이다.

## Forward Check

요구사항 열을 따라 내려가며 읽는다. 모든 요구사항 행은 적어도 하나의 덮는 테스트를 명시해야 한다.

- 코드는 있으나 테스트가 없는 요구사항은 **untested**이다 — 표시하고 `tdd-workflow` skill로 보내 변경이 완료로 간주되기 전에 테스트를 추가한다.
- 코드도 테스트도 없는 요구사항은 **unimplemented**이다 — 표시한다. 그 변경은 자신의 명세조차 충족하지 못한다.

forward 검사는 요구사항 쪽에서 본 테스트 완전성 gate다. 테스트가 얼마나 많은 coverage를 제공해야 하는지는 정의하지 않는다 — coverage 임계값은 `tdd-workflow` skill에 있다.

## Backward Check

코드에서부터 거슬러 올라가며 읽는다. 변경된 모든 함수와 새로운 모든 분기는 요구사항 행으로 되짚어져야 한다.

- **요구사항도 테스트도 없는** 코드는 orphan code다. 두 가지 중 하나로 해결한다:
  - gold-plating이나 추측성 일반화라면 **삭제한다** — 가장 저렴한 코드는 유지하지 않는 코드다.
  - 정말로 필요하지만 암묵적이었다면(에러 경로, 가드, 명세가 가정했으나 명시하지 않은 성능 제약) **derived requirement로 포착한다**. 그것에 대한 요구사항 행을 추가하여 다른 것들과 마찬가지로 추적 가능하고 테스트 가능하게 만든다. 이 derived requirement들을 명세로 되먹이는 것 자체가 하나의 산출물이다 — `do-178c` skill의 derived-requirements 규율을 참조하라.

backward 검사는 과잉 엔지니어링이 취향이 아니라 구조적으로 잡히는 지점이다.

## When to Maintain It

rigor를 criticality tier에 묶는다(`do-178c` skill의 tier 표 참조).

- **Tier A / B**: trace 매트릭스는 **required**이다. `assurance-auditor` agent가 독립 review gate의 일부로 forward와 backward 검사를 모두 감사한다.
- **Tier C and below**: 선택사항이다. 리뷰어가 혼란스러운 변경에 대해 여전히 요청할 수 있으나, gate로 강제되지는 않는다.

`assurance-auditor` agent는 이 두 검사의 감사를 소유하고, `tdd-workflow` skill은 그것들이 가리키는 테스트를 소유하며, 이 reference는 그 둘 사이의 연결만을 소유한다.
