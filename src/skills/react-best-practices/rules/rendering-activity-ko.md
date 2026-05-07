---
title: Use Activity Component for Show/Hide
impact: MEDIUM
impactDescription: preserves state/DOM
tags: rendering, activity, visibility, state-preservation
---

## 표시/숨김에는 Activity 컴포넌트를 사용한다

자주 표시 여부가 토글되는 비싼 컴포넌트의 상태와 DOM을 보존하려면 React의 `<Activity>`를 사용한다.

**사용 예:**

```tsx
import { Activity } from 'react'

function Dropdown({ isOpen }: Props) {
  return (
    <Activity mode={isOpen ? 'visible' : 'hidden'}>
      <ExpensiveMenu />
    </Activity>
  )
}
```

비싼 재렌더링과 상태 손실을 피한다.
