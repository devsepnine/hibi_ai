---
title: Use useOptimistic for Responsive Mutations
impact: MEDIUM
impactDescription: perceived latency drops to zero; automatic rollback on failure
tags: react19, actions, useOptimistic, mutations, perceived-performance
---

## Use useOptimistic for Responsive Mutations

React 19's `useOptimistic` lets a component show a provisional value
during an async action and automatically reverts if the action rejects.
Prefer it over manual "optimistic" patterns (`setState` before `await`,
rollback in `catch`) for any mutation that has a predictable end state.

Pair with Actions (`<form action={...}>` / `startTransition`) so the
hook can track pending boundaries and reconcile with the committed
server state.

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

Problems: custom rollback logic, easy to leave orphaned "pending" items
on double-submit, state diverges from server if an error is missed.

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

If `createItem` throws, React discards the optimistic state and re-renders
with the real `items`. On success, the parent (Server Component or a
mutating reducer) provides the new list, and `optimisticItems` naturally
converges back to it.

### Guidelines

- Optimistic updaters must be **pure** — no I/O, no side effects. They
  re-run on every reconciliation.
- Represent the optimistic distinction visibly (dim, spinner, disabled)
  so users understand the state isn't final.
- For delete/update, stash the prior entry so the optimistic reducer can
  compute the new list; don't rely on hidden local state.
- Don't combine `useOptimistic` with a sibling `useState` that mirrors
  the same data — pick one source of truth.

Reference: [`useOptimistic`](https://react.dev/reference/react/useOptimistic)
