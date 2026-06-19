---
title: Avoid Control Coupling (Flag Arguments Leaking Internal Structure)
impact: CRITICAL
impactDescription: control flags let a caller steer the callee's internal branches, creating the hardest-to-untangle coupling in modern code
tags: coupling, control, api-design
---

## Avoid Control Coupling (Flag Arguments Leaking Internal Structure)

Control coupling occurs when a caller passes a flag or mode argument whose only job is to select which internal branch of the callee runs. The flag leaks the callee's internal structure into the call site: the caller now has to know the callee's branches to call it correctly, and every new branch forces a new flag. Among classic coupling types, this is the most severe modern threat, because the resulting dependency is the most complex and the hardest to dissolve. Indirect leakage of hidden knowledge through flags is, in practice, worse than direct leakage.

The fix is to remove the steering: split the behavior into intention-revealing functions, or invert control with a strategy or polymorphism so the caller selects a behavior object instead of toggling internal paths. Localized strategy and broad inversion of control are both core tools of modern design.

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
