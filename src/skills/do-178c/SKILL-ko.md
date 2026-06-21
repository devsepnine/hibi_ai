---
name: do-178c
description: Pragmatic DO-178C-derived assurance methodology for general/AI-assisted development - risk-tiered rigor, bidirectional traceability, independent verification, structural coverage, and derived-requirement feedback. Use when working on safety-critical or high-blast-radius changes, assigning an assurance level, building traceability, or auditing verification rigor. 안전필수 개발, 보증수준, 위험도 티어, 양방향 추적성, 독립 검증, 구조 커버리지, 파생요구, DO-178C 개발론.
keywords: [do-178c, assurance-level, traceability, structural-coverage, derived-requirements, 보증수준, 추적성, 안전필수, 위험도티어]
---

# DO-178C Assurance Methodology

DO-178C는 항공 소프트웨어 인증 표준이지만, 그 원칙 중 세 가지는 어떤 개발에도 그대로 이식된다. 엄격성은 변경이 가진 위험에 비례해야 하고, 모든 요구사항은 그것을 충족하는 코드와 테스트로 양방향 추적되어야 하며, 작업을 검증하는 사람은 그 작업을 만든 사람이 아니어야 한다. 이 skill은 이 이식 가능한 원칙들만 가져와 실용적으로 적용한다 — 인증 문서나 항공 절차를 들여오지 않는다.

이 harness는 이 원칙들이 함축하는 게이트 대부분을 이미 소유하고 있다. 테스트와 커버리지, verification loop, 독립 리뷰 게이트, 결합 규칙, 보안 사인오프가 모두 기존 skill에 들어 있다. 그래서 이 skill은 얇은 overlay다. 새로 추가하는 것은 단 네 가지 — criticality/assurance 티어 다이얼, 양방향 추적성, 구조 커버리지 규율, 파생요구 피드백 — 뿐이고 나머지는 모두 cross-reference로 위임한다.

## When to Use

- 변경이 안전필수이거나 blast radius가 큰 경우 (auth, payments, crypto, 데이터 마이그레이션, 영속 상태, public API 계약).
- 작업에 assurance level을 부여하고 그에 맞춰 엄격성을 조절해야 하는 경우.
- 요구사항, 코드, 테스트 사이의 추적성이 필요한 경우 (forward 또는 backward).
- 검증 엄격성이 위험에 부합하는지 감사하는 경우.

D/E 티어 작업(외형, 일회성, 실험)은 이 skill을 건너뛸 수 있다 — 오버헤드가 정당화되지 않는다.

## Delegates To (DRY)

이 skill은 harness가 이미 소유한 내용을 다시 서술하지 않는다. SSOT를 가리키고 그 위에 티어 스케일링만 더한다.

| DO-178C concern | Owned by (existing SSOT) | This skill adds |
|---|---|---|
| Requirements baseline | CLAUDE.md "Problem 1-Pager" + `eval-harness` | Tier-gated baseline before A/B work |
| Requirements-based tests | `tdd-workflow` | Trace each test to a requirement |
| Coverage thresholds | `tdd-workflow` | Structural-coverage discipline per tier |
| Verification activity | `verification-loop` | Scale which checks run by tier |
| Independent review | CLAUDE.md "Mandatory post-work review" + `code-reviewer` | `assurance-auditor` + human gate for A-tier |
| Coupling / dependency direction | `dependency-design` | Tighter scrutiny at A/B |
| Security sign-off | `security-review` | Mandatory for A-tier |
| Configuration management / baselines | `commit-rules` + `pull-request` | Trace key on commits/PRs |

목표-소유자 전체 매핑은 `references/objectives-map.md`에 있다.

## Core Flow

1. **Assurance level을 분류한다 (A-E).** 실패 영향과 blast radius를 판단해 티어를 부여한다. 티어는 이후 모든 단계의 master dial이다. `references/assurance-levels.md` 참조.
2. **Requirements baseline을 수립한다.** A/B 작업의 경우 구현 전에 Problem 1-Pager(CLAUDE.md section 1)를 작성하고 acceptance eval을 확보한다 — `eval-harness` cross-ref. baseline은 추적성이 추적해 들어가는 대상이다.
3. **구현하고 derived requirement를 드러낸다.** 구현 중 spec 아래에서 도입되는 모든 동작(retry, cache, default, error code)은 derived requirement다 — 묻어두지 말고 드러낸다. 아래 Derived-Requirement Feedback 참조.
4. **양방향 traceability를 구축한다.** 모든 요구사항을 그 코드와 테스트로 forward 연결하고, 변경된 모든 unit을 요구사항으로 backward 연결한다. orphan은 표시한다. `references/traceability.md` 참조.
5. **티어 스케일 검증을 실행한다.** 활동은 `verification-loop`에, 테스트/임계값은 `tdd-workflow`에 위임하고, 그 위에 티어가 요구하는 수준의 structural coverage를 더한다. 아래 Structural Coverage 참조.
6. **독립 검증을 받는다.** CLAUDE.md section-4 post-work `code-reviewer` 게이트를 실행한다(reviewer != author). A 티어는 `assurance-auditor` 에이전트도 실행하고 human review를 요구한다.
7. **QA sign-off.** 새 게이트를 발명하지 말고 기존 게이트를 조합한다. `verification-loop`가 READY를 보고하고 `pull-request` pre-PR checklist가 통과해야 변경이 완료된 것으로 본다.

## Assurance Levels

티어는 master dial이다 — 위의 모든 단계에서 coverage, traceability, review, security의 요구 엄격성을 설정한다. 한 번, 일찍 분류하면 이후 모든 것이 거기서 읽어 간다.

| Tier | DO-178C | Failure impact | Examples | Required rigor (dials existing gates) |
|---|---|---|---|---|
| A | Catastrophic | Irreversible; data/money/security loss | auth, payments, crypto, DB migration/deletion | Max: requirements-based tests + decision/branch coverage (MC/DC aspirational); bidirectional traceability; independent `code-reviewer` + `assurance-auditor` + human review; `security-review` sign-off; full `verification-loop` |
| B | Hazardous | Major breakage, hard to reverse | core business logic, public API contracts, persistent state | High: decision/branch coverage; traceability matrix; independent `code-reviewer` (`assurance-auditor` recommended); `verification-loop` |
| C | Major | Degraded UX, recoverable | internal features, dashboards, non-critical endpoints | Standard: baseline coverage (see `tdd-workflow`); `code-reviewer`; `/verify` |
| D | Minor | Cosmetic, easily fixed | logging, copy, styling | Light: lint/type-check; review optional |
| E | No effect | Throwaway/experimental | scratch scripts, spikes | None required |

분류 가이드는 `references/assurance-levels.md` 참조.

## Structural Coverage

Structural coverage는 테스트가 코드의 어느 부분을 실제로 실행했는지 측정한다 — 테스트 완전성에 대한 점검이지 requirements-based test를 대체하지 않는다.

- **Statement coverage** — 모든 statement가 최소 한 번 실행됨.
- **Decision/branch coverage** — 모든 decision의 모든 branch가 양쪽(true와 false)으로 실행됨.
- **MC/DC (Modified Condition/Decision Coverage)** — 각 condition이 decision 결과에 독립적으로 영향을 미친다는 것이 증명됨. 이것은 A 티어의 이상이지만, 대부분의 JS/TS/Python 툴체인은 진짜 MC/DC를 측정할 수 없으므로 여기에 게이트를 걸지 않는다. aspirational로 취급한다.

위의 티어 표에 따라 statement coverage, 그다음 decision/branch coverage에 게이트를 건다. 실제 percentage는 `tdd-workflow`에 위임한다 — 숫자는 그쪽이 소유한다.

coverage가 실행되지 않은 코드를 드러내면 분류한다. **dead code**(도달 불가, 요구사항 없음)는 제거하고, **deactivated code**(의도적으로 비활성 — feature flag, guarded fallback)는 삭제하지 말고 정당화하고 문서화한다.

## Derived-Requirement Feedback

derived requirement는 spec 아래에서 도입된 동작이다 — 요구사항이 명시한 적 없지만 구현이 필요로 하는 무언가. 흔한 형태는 retry policy, cache, default value, 내부 error code, timeout이다.

규칙은 이렇다. 모든 derived requirement를 spec 소유자에게 드러내어 검토받고, 수용되면 baseline에 편입한다. 패턴은 CLAUDE.md section-3 self-improvement loop와 `MEMORY.md`를 통해 기록한다. 절대 조용히 묻어두지 않는다. 이것은 AI가 요청되지 않은 동작을 슬며시 추가하는 데 대한 직접적인 해독제다 — spec 아래의 모든 결정을 드러내도록 강제함으로써 derived-requirement feedback은 구현이 baseline에 대해 정직하게 유지되게 한다.

## Bidirectional Traceability

traceability는 양방향으로 흐른다. **Forward**: 모든 요구사항에는 그것을 구현하는 코드와 그것을 증명하는 테스트가 있어야 한다 — gap은 구축되지 않았거나 테스트되지 않은 요구사항을 뜻한다. **Backward**: 변경된 모든 unit은 요구사항으로 추적되어야 한다 — 추적 불가능한 변경은 orphan(scope creep 또는 숨겨진 derived requirement)이며 반드시 표시해야 한다.

harness가 이미 생성하는 ID를 trace key로 재사용한다. ticket ID(`commit-rules` / `pull-request`), eval 이름(`eval-harness`), test 이름(`tdd-workflow`). 병렬 ID 체계를 발명하지 않는다. `references/traceability.md` 참조.

## Deep References

- `references/assurance-levels.md` — tier 분류 가이드와 worked example.
- `references/traceability.md` — forward/backward trace 구축 및 감사.
- `references/objectives-map.md` — DO-178C 목표-harness 소유자 전체 매핑.

## Related Skills

이들에 link하고, 내용을 복제하지 않는다.

- `tdd-workflow` — requirements-based test와 coverage 임계값 (숫자를 소유).
- `verification-loop` — build/type/lint/test/security 검증 활동과 READY 게이트.
- `eval-harness` — requirements baseline을 형성하는 acceptance eval.
- `dependency-design` — coupling과 dependency-direction 점검.
- `security-review` — 보안 사인오프, A 티어에서 필수.
- `commit-rules` — commit 규약; commit에 trace key를 실어 나른다.
- `pull-request` — pre-PR checklist와 PR 규약.
- `assurance-auditor` agent / `/do-178c` command — A 티어 작업에 대한 독립 assurance 감사.
