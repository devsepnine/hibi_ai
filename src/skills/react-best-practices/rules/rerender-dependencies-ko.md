---
title: Narrow Effect Dependencies
impact: LOW
impactDescription: minimizes effect re-runs
tags: rerender, useEffect, dependencies, optimization
---

## Effect 의존성을 좁힌다

객체 대신 primitive 의존성을 명시해 effect 재실행을 최소화한다.

**잘못된 예 (user의 어떤 필드가 바뀌어도 재실행):**

```tsx
useEffect(() => {
  console.log(user.id)
}, [user])
```

**올바른 예 (id가 바뀔 때만 재실행):**

```tsx
useEffect(() => {
  console.log(user.id)
}, [user.id])
```

**파생 state는 effect 외부에서 계산한다:**

```tsx
// Incorrect: runs on width=767, 766, 765...
useEffect(() => {
  if (width < 768) {
    enableMobileMode()
  }
}, [width])

// Correct: runs only on boolean transition
const isMobile = width < 768
useEffect(() => {
  if (isMobile) {
    enableMobileMode()
  }
}, [isMobile])
```
