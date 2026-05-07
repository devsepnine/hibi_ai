---
title: Use Explicit Conditional Rendering
impact: LOW
impactDescription: prevents rendering 0 or NaN
tags: rendering, conditional, jsx, falsy-values
---

## 명시적 조건부 렌더링을 사용한다

조건이 `0`, `NaN`, 기타 렌더링되는 falsy 값일 수 있는 경우, `&&` 대신 명시적인 삼항 연산자(`? :`)를 사용해 조건부 렌더링을 작성한다.

**잘못된 예 (count가 0일 때 "0"이 렌더링됨):**

```tsx
function Badge({ count }: { count: number }) {
  return (
    <div>
      {count && <span className="badge">{count}</span>}
    </div>
  )
}

// count = 0 일 때 렌더 결과: <div>0</div>
// count = 5 일 때 렌더 결과: <div><span class="badge">5</span></div>
```

**올바른 예 (count가 0이면 아무것도 렌더링하지 않음):**

```tsx
function Badge({ count }: { count: number }) {
  return (
    <div>
      {count > 0 ? <span className="badge">{count}</span> : null}
    </div>
  )
}

// count = 0 일 때 렌더 결과: <div></div>
// count = 5 일 때 렌더 결과: <div><span class="badge">5</span></div>
```
