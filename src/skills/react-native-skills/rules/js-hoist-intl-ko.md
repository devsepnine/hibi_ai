---
title: Hoist Intl Formatter Creation
impact: LOW-MEDIUM
impactDescription: avoids expensive object recreation
tags: javascript, intl, optimization, memoization
---

## Hoist Intl Formatter Creation

`Intl.DateTimeFormat`, `Intl.NumberFormat`, `Intl.RelativeTimeFormat`을 render나
loop 안에서 생성하지 않는다. 이들은 인스턴스화 비용이 크다. locale/options가
정적이라면 모듈 스코프로 호이스팅한다.

**Incorrect (new formatter every render):**

```tsx
function Price({ amount }: { amount: number }) {
  const formatter = new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  })
  return <Text>{formatter.format(amount)}</Text>
}
```

**Correct (hoisted to module scope):**

```tsx
const currencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
})

function Price({ amount }: { amount: number }) {
  return <Text>{currencyFormatter.format(amount)}</Text>
}
```

**For dynamic locales, memoize:**

```tsx
const dateFormatter = useMemo(
  () => new Intl.DateTimeFormat(locale, { dateStyle: 'medium' }),
  [locale]
)
```

**Common formatters to hoist:**

```tsx
// Module-level formatters
const dateFormatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' })
const timeFormatter = new Intl.DateTimeFormat('en-US', { timeStyle: 'short' })
const percentFormatter = new Intl.NumberFormat('en-US', { style: 'percent' })
const relativeFormatter = new Intl.RelativeTimeFormat('en-US', {
  numeric: 'auto',
})
```

`Intl` 객체 생성은 `RegExp`나 일반 객체보다 훨씬 비싸다. 매 인스턴스화마다
locale 데이터를 파싱하고 내부 lookup table을 만들기 때문이다.
