---
title: Use useDeferredValue for Expensive Derived Renders
impact: MEDIUM
impactDescription: keeps input responsive during heavy computation
tags: rerender, useDeferredValue, optimization, concurrent
---

## 비싼 파생 렌더에는 useDeferredValue를 사용한다

사용자 입력이 비싼 계산이나 렌더를 트리거할 때, `useDeferredValue`를 사용해 입력 응답성을 유지한다. deferred value는 한 박자 늦게 따라오므로, React가 입력 업데이트를 우선 처리하고 비싼 결과는 idle 시점에 렌더할 수 있다.

**잘못된 예 (filtering 도중 입력이 끊겨 보임):**

```tsx
function Search({ items }: { items: Item[] }) {
  const [query, setQuery] = useState('')
  const filtered = items.filter(item => fuzzyMatch(item, query))

  return (
    <>
      <input value={query} onChange={e => setQuery(e.target.value)} />
      <ResultsList results={filtered} />
    </>
  )
}
```

**올바른 예 (입력은 부드럽게 유지되고, 결과는 준비되면 렌더):**

```tsx
function Search({ items }: { items: Item[] }) {
  const [query, setQuery] = useState('')
  const deferredQuery = useDeferredValue(query)
  const filtered = useMemo(
    () => items.filter(item => fuzzyMatch(item, deferredQuery)),
    [items, deferredQuery]
  )
  const isStale = query !== deferredQuery

  return (
    <>
      <input value={query} onChange={e => setQuery(e.target.value)} />
      <div style={{ opacity: isStale ? 0.7 : 1 }}>
        <ResultsList results={filtered} />
      </div>
    </>
  )
}
```

**언제 사용하는가:**

- 큰 리스트의 filtering/searching
- 입력에 반응하는 비싼 시각화 (charts, graphs)
- 눈에 띄는 렌더 지연을 일으키는 모든 파생 state

**참고:** 비싼 계산은 deferred value를 의존성으로 하는 `useMemo`로 감싼다. 그렇지 않으면 매 렌더마다 다시 실행된다.

참고: [React useDeferredValue](https://react.dev/reference/react/useDeferredValue)
