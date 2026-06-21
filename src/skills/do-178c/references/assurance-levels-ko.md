# Assurance Levels (DAL A–E) — Classification Guide

assurance 티어는 `do-178c` 오버레이의 마스터 다이얼이다. 한 번 설정하면 기존의 모든 게이트 — 커버리지 깊이, 리뷰 독립성, security 사인오프, 검증 범위 — 가 그에 맞춰 조정된다. 오버레이의 나머지 모든 것은 이 하나의 결정에서 파생된다.

분류 기준은 **실패 시 최악의 폭발 반경(worst-case blast radius)**이며, 투입 노력이나 코드 크기, 작업의 체감 난이도가 아니다. auth 검사에 대한 세 줄짜리 변경은 Tier A이고, 2,000줄짜리 대시보드 리팩토링은 Tier C일 수 있다. 얼마나 많이 입력했는지가 아니라, 코드가 틀린 채로 배포되면 무엇이 깨지는지를 물어라.

## Canonical Tier Table

| Tier | DO-178C | Failure impact | Examples | Required rigor (dials existing gates) |
|---|---|---|---|---|
| A | Catastrophic | Irreversible; data/money/security loss | auth, payments, crypto, DB migration/deletion | Max: requirements-based tests + decision/branch coverage (MC/DC aspirational); bidirectional traceability; independent `code-reviewer` + `assurance-auditor` + human review; `security-review` sign-off; full `verification-loop` |
| B | Hazardous | Major breakage, hard to reverse | core business logic, public API contracts, persistent state | High: decision/branch coverage; traceability matrix; independent `code-reviewer` (`assurance-auditor` recommended); `verification-loop` |
| C | Major | Degraded UX, recoverable | internal features, dashboards, non-critical endpoints | Standard: baseline coverage (see `tdd-workflow`); `code-reviewer`; `/verify` |
| D | Minor | Cosmetic, easily fixed | logging, copy, styling | Light: lint/type-check; review optional |
| E | No effect | Throwaway/experimental | scratch scripts, spikes | None required |

## How to Classify

짧은 결정 절차:

1. 질문한다: **"이것이 틀린 채로 배포되면 일어날 수 있는 최악의 일은 무엇인가?"**
2. 그 심각도를 티어에 매핑한다:
   - 데이터, 돈, 또는 보안 태세의 되돌릴 수 없는 손실 → **A**
   - 되돌리기 어려운 major breakage → **B**
   - 저하되었지만 복구 가능한 사용자 경험 → **C**
   - 사소하고 쉽게 고칠 수 있음 → **D**
   - 사용자에게 보이는 영향 없음, 일회성 → **E**
3. **불확실하면 한 티어 올림(round up)한다.** C를 B로 과도하게 보증하는 비용은 작지만, A를 C로 과소 보증하는 비용은 바로 이 오버레이가 막으려는 실패 모드다.
4. **작업 도중 범위가 바뀌면 재분류한다.** "대시보드 수정"이 auth 경로를 건드리도록 커지면, 멈추고 다시 티어를 매긴다 — 새 게이트가 변경 전체에 적용된다.

## What Each Tier Requires

각 티어는 기존 게이트를 조정할 뿐, 새로운 절차를 도입하지 않는다. 실제 메커니즘은 해당 게이트를 소유한 스킬을 읽어라.

- **Tier A** — 최대 rigor. `tdd-workflow`의 최고 커버리지 기준에서 requirements-based tests를 수행하고, 각 요구사항을 테스트와 코드로 연결하는 bidirectional traceability를 갖춘다. 최대 깊이의 `verification-loop`. `code-reviewer`와 `assurance-auditor` 양쪽의 독립 리뷰, 그리고 human gate. 필수 `security-review` 사인오프. 실패가 전파되지 못하도록 가장 엄격한 `dependency-design` 결합도 규율.
- **Tier B** — 높은 rigor. `tdd-workflow`에 따른 decision/branch coverage, traceability matrix, 그리고 전체 `verification-loop`. 독립 `code-reviewer`, `assurance-auditor` 권장. blast radius를 가두기 위해 `dependency-design`을 적용한다.
- **Tier C** — 표준 rigor. `tdd-workflow`의 baseline coverage, `code-reviewer`, 그리고 `verification-loop`의 `/verify`. 이것이 일상적인 기본값이다.
- **Tier D** — 가벼운 수준. lint와 type-check만, 리뷰는 선택. 오버헤드가 거의 없다 — logging이나 copy 변경에 의식(ceremony)을 만들어내지 마라.
- **Tier E** — 요구사항 없음. 일회성·실험 작업에는 보증 의무가 없다. 설계상 오버헤드가 거의 없다.

여기서 커버리지 수치, 검증 단계, 리뷰 체크리스트를 다시 기술하지 마라 — 각각 `tdd-workflow`, `verification-loop`, `code-reviewer`, `security-review`, `dependency-design`에 있다. 티어는 그중 어느 것을 얼마나 엄격하게 적용할지만 선택한다.

## Declaring the Tier

리뷰어가 어떤 게이트 세트를 확인해야 하는지 알 수 있도록, 작업 시작 시점에 티어를 명시한다:

- 계획이 필요한 작업의 경우 **plan**(`CLAUDE.md` section 1의 `Problem 1-Pager`)에 명시.
- 리뷰어가 올바른 게이트가 실행되었는지 검증하도록 **PR description**(`pull-request` 참조)에 명시.
- baseline 기록을 위해 **commit context**(`commit-rules` 참조)에 명시.

티어를 기록하는 것이 보증을 감사 가능하게(auditable) 만든다: 리뷰어는 A 티어 변경이 단지 `code-reviewer`만이 아니라 실제로 `assurance-auditor`와 `security-review`를 거쳤는지 확인할 수 있다.

## Independence by Tier

독립성(independence)은 리뷰어가 작성자가 아님을 의미한다. 이는 `CLAUDE.md` section 4의 기존 "Mandatory post-work review" 게이트에 매핑된다 — 티어는 몇 명의 독립적인 시선이 필요한지만 설정한다. 그 게이트를 중복하지 말고, 조정만 하라.

| Tier | Independent review required |
|---|---|
| A | Separate `assurance-auditor` **and** human review (on top of `code-reviewer`) |
| B | Independent `code-reviewer` (`assurance-auditor` recommended) |
| C | `code-reviewer` |
| D | Optional |
| E | Optional |
