---
title: Put Interaction Logic in Event Handlers
impact: MEDIUM
impactDescription: avoids effect re-runs and duplicate side effects
tags: rerender, useEffect, events, side-effects, dependencies
---

## 상호작용 로직은 이벤트 핸들러에 둔다

특정 사용자 액션(submit, click, drag)이 트리거하는 사이드 이펙트는 그 이벤트 핸들러에서 실행한다. 액션을 state + effect로 모델링하지 않는다 — 그러면 무관한 변경에 effect가 재실행되거나 액션이 중복 발생할 수 있다.

**잘못된 예 (이벤트를 state + effect로 모델링):**

```tsx
function Form() {
  const [submitted, setSubmitted] = useState(false)
  const theme = useContext(ThemeContext)

  useEffect(() => {
    if (submitted) {
      post('/api/register')
      showToast('Registered', theme)
    }
  }, [submitted, theme])

  return <button onClick={() => setSubmitted(true)}>Submit</button>
}
```

**올바른 예 (핸들러에서 처리):**

```tsx
function Form() {
  const theme = useContext(ThemeContext)

  function handleSubmit() {
    post('/api/register')
    showToast('Registered', theme)
  }

  return <button onClick={handleSubmit}>Submit</button>
}
```

참고: [Should this code move to an event handler?](https://react.dev/learn/removing-effect-dependencies#should-this-code-move-to-an-event-handler)
