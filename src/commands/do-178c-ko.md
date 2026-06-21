---
description: 'Classify a task or module assurance tier (A-E) and apply DO-178C-derived rigor - traceability, tier-scaled verification, independent audit - by delegating to existing gates'
argument-hint: "[path|task] [--tier A|B|C|D|E]"
allowed-tools: Read, Grep, Glob
model: sonnet
effort: high
---

# Do-178c Command

## What This Command Does

`do-178c` 스킬을 로드하고, `$ARGUMENTS`로 지정된 대상(경로 또는 작업 설명)에 그 보증 방법론을 적용합니다. 해당 대상에 대해 다음을 수행합니다:

- **(a) 보증 tier(A-E) 할당 또는 사용** — 변경의 최악 시나리오 영향 범위(blast radius)를 기준으로 합니다.
- **(b) 해당 tier가 조절하는 gate 보고** — `tdd-workflow`의 coverage 깊이, `verification-loop`의 검증 범위, `code-reviewer`와 `assurance-auditor`의 리뷰 독립성, 그리고 `security-review` sign-off.
- **(c) 양방향 traceability와 structural-coverage 기대치 점검** — 모든 요구사항이 테스트로, 그리고 테스트가 요구사항으로 매핑되며, 고아(orphan) 요구사항이나 코드 경로가 없어야 합니다.

이미 criticality를 알고 있어 분류를 건너뛰고 싶을 때는 선택적 `--tier A|B|C|D|E` 오버라이드를 따릅니다.

## When to Use

- 안전이 중요하거나 영향 범위가 큰 변경(auth, payments, crypto, DB migration)을 시작할 때, 필요한 rigor를 사전에 고정하고자 할 때.
- 반사적으로 과도하거나 부족하게 테스트하는 대신, 변경에 실제로 필요한 rigor 수준을 결정할 때.
- 완료를 선언하기 전에, 끝난 변경이 해당 tier의 기준을 충족했는지 감사(audit)할 때.

## How It Works

1. **Tier 분류** — 최악 시나리오 영향 범위를 기준으로 합니다. 무엇이 깨지는지, 얼마나 되돌리기 어려운지, 무엇을 건드리는지(data, money, security, persistent state). `--tier`가 주어지면 그것을 사용하고, 아니면 대상에서 추론합니다. 이 명령이 소유하는 유일한 신규 의사결정입니다.
2. **조절되는 gate 나열** — 해당 tier에 맞춰: `tdd-workflow`의 coverage 목표, `verification-loop`의 검증 깊이, 리뷰가 독립적이어야 하는지(`code-reviewer`, 그리고 상위 tier는 `assurance-auditor`), `security-review` sign-off가 필수인지.
3. **Traceability와 coverage 점검** — 양방향 traceability(각 요구사항에 이를 커버하는 테스트가 있고, 각 테스트가 요구사항으로 추적됨)와 structural coverage가 tier 기대치를 충족하는지 확인합니다. statement에서 decision/branch 순으로 봅니다. MC/DC는 A-tier의 이상향이지만 gate로 두지 않습니다. 대부분의 JS/TS/Python 스택은 진정한 MC/DC를 측정할 수 없기 때문입니다.
4. **A-tier는 `assurance-auditor`로 핸드오프** — 독립 audit을 받습니다. 작성자가 catastrophic-impact 작업을 스스로 인증할 수 없기 때문입니다.

## Example Usage

```
User: /do-178c src/payments

Agent: Loaded `do-178c`. Classified src/payments as Tier A (Catastrophic) —
handles money movement and persistent ledger state; failures are irreversible.

Dialed gates for Tier A:
  - tdd-workflow: requirements-based tests + decision/branch coverage
    (MC/DC aspirational, not gated)
  - verification-loop: full (build, type, lint, test, security)
  - code-reviewer + assurance-auditor + human review (independent)
  - security-review: sign-off required

Traceability + coverage check:
  - REQ "refunds must be idempotent" has NO covering test — untested
    requirement; add a requirements-based test before merge.
  - reconcileLedger() in processor.ts:87 traces to no requirement —
    orphan function; either link it to a requirement or remove it.

Next: handing off to `assurance-auditor` for the independent A-tier audit.
```

## Related Commands

- `/tdd` — tier가 요구하는 requirements-based 테스트와 coverage를 진행합니다.
- `/verify` — 해당 tier의 build/type/lint/test 검증 깊이를 실행합니다.
- `/code-review` — diff에 대한 독립적인 품질 및 보안 리뷰.
- `/deps` — 변경된 경계의 의존성 방향과 coupling을 확인합니다.
- `/security-review` — A-tier(auth, payments, secrets, input)에 필수인 sign-off.

Full methodology lives in the `do-178c` skill — follow that as the source of truth.
