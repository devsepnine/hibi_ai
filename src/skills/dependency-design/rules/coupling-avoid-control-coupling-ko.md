---
title: Avoid Control Coupling (Flag Arguments Leaking Internal Structure)
impact: CRITICAL
impactDescription: control flags let a caller steer the callee's internal branches, creating the hardest-to-untangle coupling in modern code
tags: coupling, control, api-design
---

## Avoid Control Coupling (Flag Arguments Leaking Internal Structure)

Control coupling은 호출자가 오직 피호출자의 어떤 내부 분기를 실행할지 고르기 위한 flag나 mode 인자를 넘길 때 발생한다. 이 flag는 피호출자의 내부 구조를 호출 지점으로 유출시킨다. 즉, 호출자가 올바르게 호출하려면 피호출자의 분기들을 알아야 하고, 분기가 하나 늘 때마다 새로운 flag가 강요된다. 고전적인 결합 유형 중에서 이것이 가장 심각한 현대적 위협이다. 그 결과 생기는 의존성이 가장 복잡하고 해소하기 가장 어렵기 때문이다. 실제로 flag를 통한 은닉 지식의 간접 누출은 직접 누출보다 더 나쁘다.

해결책은 조종 자체를 제거하는 것이다. 동작을 의도를 드러내는 함수들로 분리하거나, strategy 혹은 polymorphism으로 제어를 역전시켜 호출자가 내부 경로를 토글하는 대신 동작 객체를 선택하게 한다. 지엽적인 strategy와 전체적인 inversion of control은 모두 현대 설계의 핵심 도구다.

**Incorrect:**

```ts
// The boolean flags drive a switch on the callee's internal structure.
// Callers must know every branch to call this correctly.
function generateReport(
  data: SalesRow[],
  isAdmin: boolean,
  exportAsPdf: boolean,
): Buffer | string {
  let rows = data
  if (isAdmin) {
    rows = data // admins see raw rows including margins
  } else {
    rows = data.map((r) => ({ ...r, margin: undefined }))
  }

  if (exportAsPdf) {
    return renderPdf(rows)
  } else {
    return renderCsv(rows)
  }
}

// Call sites are unreadable and must track flag order/meaning.
generateReport(rows, true, false)
generateReport(rows, false, true)
```

**Correct:**

```ts
// Intention-revealing functions: no flag steers the internals.
function adminRows(data: SalesRow[]): SalesRow[] {
  return data
}
function viewerRows(data: SalesRow[]): SalesRow[] {
  return data.map((r) => ({ ...r, margin: undefined }))
}

// Strategy object: the caller injects the format behavior
// instead of toggling an internal branch.
interface ReportFormat {
  render(rows: SalesRow[]): Buffer | string
}
const PdfFormat: ReportFormat = { render: (rows) => renderPdf(rows) }
const CsvFormat: ReportFormat = { render: (rows) => renderCsv(rows) }

function generateReport(rows: SalesRow[], format: ReportFormat) {
  return format.render(rows)
}

// Call sites read as intent, and new formats add no flags.
generateReport(adminRows(rows), CsvFormat)
generateReport(viewerRows(rows), PdfFormat)
```

Reference: [Coupling types and threat ranking](../references/coupling-models.md)
