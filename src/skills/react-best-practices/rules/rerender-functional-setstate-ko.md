---
title: Use Functional setState Updates
impact: MEDIUM
impactDescription: prevents stale closures and unnecessary callback recreations
tags: react, hooks, useState, useCallback, callbacks, closures
---

## Functional setState 업데이트를 사용한다

현재 state 값에 기반해 state를 갱신할 때는 state 변수를 직접 참조하는 대신 setState의 functional update form을 사용한다. 이는 stale closure를 방지하고, 불필요한 의존성을 제거하며, 안정적인 콜백 참조를 만든다.

**잘못된 예 (state를 의존성으로 요구):**

```tsx
function TodoList() {
  const [items, setItems] = useState(initialItems)
  
  // Callback must depend on items, recreated on every items change
  const addItems = useCallback((newItems: Item[]) => {
    setItems([...items, ...newItems])
  }, [items])  // items dependency causes recreations
  
  // Risk of stale closure if dependency is forgotten
  const removeItem = useCallback((id: string) => {
    setItems(items.filter(item => item.id !== id))
  }, [])  // Missing items dependency - will use stale items!
  
  return <ItemsEditor items={items} onAdd={addItems} onRemove={removeItem} />
}
```

첫 번째 콜백은 `items`가 변경될 때마다 재생성되며, 자식 컴포넌트가 불필요하게 재렌더될 수 있다. 두 번째 콜백은 stale closure 버그가 있어 항상 초기 `items` 값을 참조한다.

**올바른 예 (안정적인 콜백, stale closure 없음):**

```tsx
function TodoList() {
  const [items, setItems] = useState(initialItems)
  
  // Stable callback, never recreated
  const addItems = useCallback((newItems: Item[]) => {
    setItems(curr => [...curr, ...newItems])
  }, [])  // No dependencies needed
  
  // Always uses latest state, no stale closure risk
  const removeItem = useCallback((id: string) => {
    setItems(curr => curr.filter(item => item.id !== id))
  }, [])  // Safe and stable
  
  return <ItemsEditor items={items} onAdd={addItems} onRemove={removeItem} />
}
```

**이점:**

1. **안정적인 콜백 참조** — state가 변해도 콜백을 재생성하지 않는다
2. **stale closure 없음** — 항상 최신 state 값으로 동작한다
3. **의존성 감소** — 의존성 배열을 단순화하고 메모리 누수를 줄인다
4. **버그 예방** — React closure 관련 가장 흔한 버그 원인을 제거한다

**Functional update를 써야 할 때:**

- 현재 state 값에 의존하는 모든 setState
- state가 필요한 useCallback/useMemo 내부
- state를 참조하는 이벤트 핸들러
- state를 갱신하는 비동기 연산

**직접 업데이트가 적절할 때:**

- state를 정적 값으로 설정: `setCount(0)`
- props/인자만으로 state 설정: `setName(newName)`
- state가 이전 값에 의존하지 않을 때

**참고:** 프로젝트에 [React Compiler](https://react.dev/learn/react-compiler)가 활성화되어 있다면 컴파일러가 일부 케이스를 자동 최적화하지만, 정확성과 stale closure 버그 방지를 위해 functional update가 여전히 권장된다.
