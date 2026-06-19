---
title: Structure Code for AI Partial Ownership
impact: HIGH
impactDescription: code an AI can edit by loading only the task-relevant subset stays changeable; code that drags in a web of cyclic files does not
tags: ai, ownership, context
---

## Structure Code for AI Partial Ownership

AI 이전에는 사람이 코드베이스의 고정된 한 부분을 소유했고, 소프트웨어 공학은 대체로 그 소유자들 사이의 커뮤니케이션 비용을 줄이는 기술이었다. AI 에이전트는 그런 의미의 소유를 하지 않는다. 저장소 전체를 머릿속에 담지 않는다. 주어진 지시마다 세션 context window에 들어가는 만큼만 분석하고, 행동에 필요한 만큼의 이해만 재구성한 뒤 나머지는 버린다.

이는 핵심 설계 질문을 다시 정의한다. 코드가 *목적별로 부분 소유*될 수 있도록 구조화되어 있는가? 에이전트가 격리된 모듈 하나를 로딩해 온전히 이해하고 수정할 수 있을 때 변경 비용은 낮다. 파일 하나를 건드리는 데에도 올바른 멘탈 모델을 세우기 위해 cyclic하게 결합된 파일의 그물을 끌어와야 한다면 변경 비용은 높다 — 게다가 그 context가 애초에 다 들어가지 않을 수도 있다.

선호 순서대로 정리한 최적의 목표는 다음과 같다.

1. 지시 단위로 완전히 격리된 모듈 — 기능이나 수정이 독립적으로 로딩하고 편집할 수 있는 모듈 안에 존재한다.
2. 연결이 불가피하다면, unidirectional dependency의 graph 구조 — 에이전트가 의존성을 한 방향으로만 따라가고 closure가 제한된 상태로 유지된다.
3. 작업에 필요한 만큼만 로딩 — 최소 closure가 context window에 들어갈 만큼 작게 유지된다.

Cyclic coupling은 이 세 가지를 모두 무너뜨린다. 최소 closure 자체가 존재하지 않으므로, 어느 한 부분을 안전하게 변경하려면 에이전트가 클러스터 전체를 로딩해야 한다.

**Incorrect:**

```typescript
// invoice.ts — to understand invoicing you must load customer.ts...
import { Customer } from "./customer"

export class Invoice {
  constructor(public customer: Customer) {}
  recalculate() {
    this.customer.refreshOutstandingBalance() // reaches back into Customer
  }
}

// customer.ts — ...which reaches back into invoice.ts (cycle)
import { Invoice } from "./invoice"

export class Customer {
  invoices: Invoice[] = []
  refreshOutstandingBalance() {
    this.invoices.forEach((i) => i.recalculate()) // back into Invoice
  }
}
// Editing either file requires loading BOTH (and whatever they each pull in).
// The "minimal context" for a one-line change is the entire cluster.
```

**Correct:**

```typescript
// invoice.ts — a self-contained module, editable in isolation.
// It depends on plain data, not on a class that depends back on it.
export interface InvoiceLine {
  amount: number
}

export function invoiceTotal(lines: InvoiceLine[]): number {
  return lines.reduce((sum, line) => sum + line.amount, 0)
}

// balance.ts — depends on invoice.ts in ONE direction only.
// To change balance logic, load balance.ts (+ the small invoice.ts type).
// To change invoice math, load invoice.ts ALONE — no cycle to drag in.
import { type InvoiceLine, invoiceTotal } from "./invoice"

export function outstandingBalance(openInvoices: InvoiceLine[][]): number {
  return openInvoices.reduce((sum, lines) => sum + invoiceTotal(lines), 0)
}
```

수정된 버전에서는 에이전트가 `invoice.ts` 하나만 로딩해 청구 계산 로직을 변경할 수 있으며, context가 최소이고 제한되어 있다 — 이것이 바로 AI가 의존하는 부분 소유다.

Reference: [Dependency direction](../references/monorepo.md), [Isolation by responsibility](../references/complexity.md)
