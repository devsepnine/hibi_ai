---
title: Suppress Expected Hydration Mismatches
impact: LOW-MEDIUM
impactDescription: avoids noisy hydration warnings for known differences
tags: rendering, hydration, ssr, nextjs
---

## 예상된 Hydration 불일치는 억제한다

SSR 프레임워크(예: Next.js)에서는 일부 값이 의도적으로 서버와 클라이언트에서 다르다 (랜덤 ID, 날짜, 로케일/타임존 포맷팅). 이러한 *예상된* 불일치에 대해서는 동적 텍스트를 `suppressHydrationWarning`이 적용된 요소로 감싸 시끄러운 경고를 방지한다. 실제 버그를 숨기는 용도로 사용하지 않는다. 남용하지 않는다.

**잘못된 예 (알려진 불일치 경고):**

```tsx
function Timestamp() {
  return <span>{new Date().toLocaleString()}</span>
}
```

**올바른 예 (예상된 불일치만 억제):**

```tsx
function Timestamp() {
  return (
    <span suppressHydrationWarning>
      {new Date().toLocaleString()}
    </span>
  )
}
```
