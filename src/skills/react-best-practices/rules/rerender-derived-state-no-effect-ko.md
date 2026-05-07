---
title: Calculate Derived State During Rendering
impact: MEDIUM
impactDescription: avoids redundant renders and state drift
tags: rerender, derived-state, useEffect, state
---

## 파생 state는 렌더링 중에 계산한다

현재의 props/state로부터 계산할 수 있는 값이라면, state에 저장하거나 effect에서 갱신하지 않는다. 추가 렌더링과 state drift를 피하기 위해 렌더링 중에 derive한다. 단지 prop 변경에 반응하기 위해 effect에서 state를 설정하지 않는다. 대신 derived value 또는 keyed reset을 우선 사용한다.

**잘못된 예 (불필요한 state와 effect):**

```tsx
function Form() {
  const [firstName, setFirstName] = useState('First')
  const [lastName, setLastName] = useState('Last')
  const [fullName, setFullName] = useState('')

  useEffect(() => {
    setFullName(firstName + ' ' + lastName)
  }, [firstName, lastName])

  return <p>{fullName}</p>
}
```

**올바른 예 (렌더링 중에 derive):**

```tsx
function Form() {
  const [firstName, setFirstName] = useState('First')
  const [lastName, setLastName] = useState('Last')
  const fullName = firstName + ' ' + lastName

  return <p>{fullName}</p>
}
```

참고: [You Might Not Need an Effect](https://react.dev/learn/you-might-not-need-an-effect)
