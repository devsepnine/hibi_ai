---
title: Subscribe to Derived State
impact: MEDIUM
impactDescription: reduces re-render frequency
tags: rerender, derived-state, media-query, optimization
---

## 파생 state를 구독한다

연속적인 값이 아닌 파생 boolean state를 구독해 재렌더 빈도를 줄인다.

**잘못된 예 (모든 픽셀 변화마다 재렌더):**

```tsx
function Sidebar() {
  const width = useWindowWidth()  // updates continuously
  const isMobile = width < 768
  return <nav className={isMobile ? 'mobile' : 'desktop'} />
}
```

**올바른 예 (boolean이 변할 때만 재렌더):**

```tsx
function Sidebar() {
  const isMobile = useMediaQuery('(max-width: 767px)')
  return <nav className={isMobile ? 'mobile' : 'desktop'} />
}
```
