---
title: Split Combined Hook Computations
impact: MEDIUM
impactDescription: avoids recomputing independent steps
tags: rerender, useMemo, useEffect, dependencies, optimization
---

## 결합된 Hook 계산을 분리한다

하나의 hook이 서로 다른 의존성을 가진 여러 독립 작업을 포함할 때는 별도의 hook으로 분리한다. 결합된 hook은 의존성 중 하나라도 변경되면 모든 작업을 재실행한다 — 일부 작업이 변경된 값을 사용하지 않더라도 마찬가지다.

**잘못된 예 (`sortOrder` 변경이 filtering도 재계산하게 만듦):**

```tsx
const sortedProducts = useMemo(() => {
  const filtered = products.filter((p) => p.category === category)
  const sorted = filtered.toSorted((a, b) =>
    sortOrder === "asc" ? a.price - b.price : b.price - a.price
  )
  return sorted
}, [products, category, sortOrder])
```

**올바른 예 (filtering은 products나 category가 변경될 때만 재계산):**

```tsx
const filteredProducts = useMemo(
  () => products.filter((p) => p.category === category),
  [products, category]
)

const sortedProducts = useMemo(
  () =>
    filteredProducts.toSorted((a, b) =>
      sortOrder === "asc" ? a.price - b.price : b.price - a.price
    ),
  [filteredProducts, sortOrder]
)
```

이 패턴은 무관한 사이드 이펙트를 결합한 `useEffect`에도 적용된다.

**잘못된 예 (의존성 중 하나만 바뀌어도 두 effect 모두 실행):**

```tsx
useEffect(() => {
  analytics.trackPageView(pathname)
  document.title = `${pageTitle} | My App`
}, [pathname, pageTitle])
```

**올바른 예 (effect가 독립적으로 실행):**

```tsx
useEffect(() => {
  analytics.trackPageView(pathname)
}, [pathname])

useEffect(() => {
  document.title = `${pageTitle} | My App`
}, [pageTitle])
```

**참고:** 프로젝트에 [React Compiler](https://react.dev/learn/react-compiler)가 활성화되어 있다면, 컴파일러가 의존성 추적을 자동 최적화하며 일부 케이스를 처리해 줄 수 있다.
