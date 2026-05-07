---
title: Use useOptimistic for Responsive Mutations
impact: MEDIUM
impactDescription: perceived latency drops to zero; automatic rollback on failure
tags: react19, actions, useOptimistic, mutations, perceived-performance
---

## Use useOptimistic for Responsive Mutations

React 19의 `useOptimistic`은 비동기 액션이 진행되는 동안 컴포넌트가 잠정 값을 보여주고, 액션이 실패하면 자동으로 되돌린다.
종료 상태가 예측 가능한 mutation이라면, 수동 "낙관적 업데이트" 패턴(`await` 전에 `setState` 후 `catch`에서 롤백)보다 이 hook을 사용한다.

Action(`<form action={...}>` / `startTransition`)과 함께 사용하면, hook이 pending 경계를 추적하고 커밋된 서버 상태와 자동으로 화해(reconcile)한다.

### Incorrect — manual optimistic-then-rollback

```tsx
const [items, setItems] = useState(initial)

async function addItem(name: string) {
  const temp = { id: Math.random(), name, pending: true }
  setItems((prev) => [...prev, temp])
  try {
    const saved = await createItem(name)
    setItems((prev) => prev.map((it) => (it.id === temp.id ? saved : it)))
  } catch (err) {
    setItems((prev) => prev.filter((it) => it.id !== temp.id))
    showToast(err.message)
  }
}
```

문제점: 커스텀 롤백 로직, 더블 서밋 시 고아 "pending" 항목이 남기 쉬움, 에러를 놓치면 상태가 서버와 어긋남.

### Correct — useOptimistic inside an Action

```tsx
function ItemList({ items }: { items: Item[] }) {
  const [optimisticItems, addOptimistic] = useOptimistic(
    items,
    (state, pending: Item) => [...state, { ...pending, pending: true }],
  )

  async function addAction(formData: FormData) {
    const name = String(formData.get('name'))
    addOptimistic({ id: crypto.randomUUID(), name })
    await createItem(name) // server revalidates, parent re-passes new items
  }

  return (
    <form action={addAction}>
      <input name="name" />
      <button type="submit">Add</button>
      <ul>
        {optimisticItems.map((it) => (
          <li key={it.id} style={{ opacity: it.pending ? 0.5 : 1 }}>
            {it.name}
          </li>
        ))}
      </ul>
    </form>
  )
}
```

`createItem`이 throw하면 React가 낙관적 상태를 폐기하고 실제 `items`로 다시 렌더링한다. 성공하면 부모(Server Component 또는 mutating reducer)가 새 리스트를 제공하고, `optimisticItems`는 자연스럽게 그 값으로 수렴한다.

### Guidelines

- 낙관적 업데이트 함수는 반드시 **순수 함수**여야 한다 — I/O 없음, 부수효과 없음. 화해 시마다 재실행된다.
- 낙관적인 상태는 시각적으로 구분해 보여준다(흐림, 스피너, disabled). 사용자가 아직 확정되지 않은 상태임을 알 수 있도록.
- 삭제/수정의 경우 이전 항목을 어딘가에 보관해 두어 낙관적 reducer가 새 리스트를 계산할 수 있게 한다. 숨겨진 local state에 의존하지 않는다.
- 같은 데이터를 미러링하는 형제 `useState`와 `useOptimistic`을 함께 쓰지 않는다 — 단일 진실 공급원을 선택한다.

Reference: [`useOptimistic`](https://react.dev/reference/react/useOptimistic)
