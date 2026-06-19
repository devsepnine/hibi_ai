---
description: 'Audit dependency direction and coupling of a module/path and report violations by threat ranking with fixes'
argument-hint: "[path|module]"
allowed-tools: Read, Grep, Glob
model: sonnet
effort: high
---

# Deps Command

## What This Command Does

`dependency-design` skill을 로드하고, `$ARGUMENTS`로 지정한 대상(경로 또는 모듈)의 의존성 방향과 결합도를 감사합니다. 대상과 그 import들을 정적으로 스캔하여 다음 결합 위반을 찾아냅니다.

- **Control coupling / flag arguments** — 호출자가 boolean이나 모드 flag를 넘겨서 피호출자가 실행할 분기를 선택하게 만드는 경우. 호출자가 피호출자의 내부를 알고 있다는 뜻입니다.
- **Cyclic or bidirectional imports** — module A가 B를 import하고 B가 다시 A를 import하는 경우(직접 또는 순환을 통해). 어느 쪽도 독립적으로 이해하거나 변경할 수 없습니다.
- **Implementation-knowledge leaks** — 소비자가 안정적인 contract 대신 private 내부, 구체 타입, 가정된 데이터 형태에 손을 뻗어 public surface를 관통하는 경우.
- **Shared-resource / singleton coupling** — 모듈들이 명시적 파라미터 대신 공유 가변 global, singleton, ambient state를 통해 조율하는 경우.
- **lib -> lib monorepo violations** — 한 library 패키지가 의존해서는 안 되는 형제 library를 import하여 의도된 의존성 그래프와 계층을 깨뜨리는 경우.
- **Abstraction-level inconsistencies** — 고수준 정책 모듈이 저수준 메커니즘 세부에 의존하여 의도된 의존성 방향을 뒤집는 경우.

## When to Use

- 모듈을 추출, 분리, 병합하기 전에 의존성이 한 방향을 향하는지 확인할 때.
- 한 파일의 변경이 무관한 파일들의 수정을 계속 강제할 때(결합 smell).
- 새 패키지 경계나 monorepo `lib`를 리뷰하며 의존성 그래프를 강제할 때.
- circular-import나 build-order 오류가 cycle을 암시할 때.

## How It Works

1. **각 발견 항목을 분류**합니다 — module-coupling 위협 순위(Control > External > Common > Contents > Stamp > Data, 나쁜 것부터 좋은 것 순)와 Cynefin context(Clear / Complicated / Complex / Chaotic)로 분류하여, 결합 유형과 도메인 이해도를 함께 반영한 severity를 매깁니다.
2. **각 위반을 보고**합니다 — `file:line`, 결합 유형, 순위, 부여된 severity, 그리고 양쪽을 왜 결합시키는지에 대한 한 줄 설명과 함께 보고합니다.
3. **최소 수정을 제안**합니다 — 단방향, contract 기반 의존성으로 향하는 수정: 의존성 역전, flag 기반 함수를 명명된 함수들로 분리, 안정적인 interface 도입, state를 명시적으로 전달 등 결합을 제거하는 가장 작은 변경을 제시합니다.

## Example Usage

```
User: /deps src/payments

Agent: Loaded `dependency-design`. Scanned src/payments (11 files).

CRITICAL (Control coupling, Complicated)
  src/payments/processor.ts:42 — charge(order, isRefund) branches on the
  isRefund flag; callers steer internal logic.
  Fix: split into charge(order) and refund(order); no shared flag.

HIGH (Cyclic import, Complex)
  src/payments/ledger.ts:8 imports ./processor, and
  src/payments/processor.ts:5 imports ./ledger.
  Fix: extract the shared LedgerEntry contract into ledger/types.ts;
  have both depend on it one-way.

MEDIUM (lib -> lib violation, Clear)
  src/payments/index.ts:3 imports @repo/billing-ui (UI lib).
  Fix: invert — billing-ui depends on payments, not the reverse.

No Stamp/Data downgrades needed elsewhere.
```

## Related Agents

- `architect` — 위반이 확인된 뒤 경계를 재설계할 때.
- `code-reviewer` — 새 diff에서 결합 회귀를 잡아낼 때.

## Related Commands

- `/code-review` — 커밋되지 않은 변경의 품질과 보안을 리뷰합니다.
- `/verify` — 수정 후에도 코드베이스가 여전히 빌드되고 통과하는지 확인합니다.

전체 방법론은 `dependency-design` skill에 있습니다 — 이를 source of truth로 따르십시오.
