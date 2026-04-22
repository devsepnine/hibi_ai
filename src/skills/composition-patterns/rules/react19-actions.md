---
title: Compose Forms Around Actions, Not Local Loading State
impact: MEDIUM
impactDescription: removes prop-drilled pending/error boilerplate; enables progressive enhancement
tags: react19, actions, forms, useActionState, useFormStatus, composition
---

## Compose Forms Around Actions, Not Local Loading State

React 19 promotes **Actions** — async functions passed to `<form action={fn}>`
or `<button formAction={fn}>` — as the primary composition boundary for
mutations. Combined with `useActionState`, `useFormStatus`, and
`useOptimistic`, they replace hand-rolled `isLoading` / `error` /
`startTransition` plumbing that used to be threaded through every descendant.

Use Actions when the mutation is **form-shaped** (user submits, server
updates, UI reflects). Stick with classic `onClick` handlers only for
side-effects that aren't really mutations (opening a modal, logging).

### Benefits over manual state

- **Transitions built-in**: pending state is managed by the form boundary.
- **Progressive enhancement**: a plain `<form action="/path">` still works
  if JS is disabled, because Actions reuse the native form contract.
- **Composable**: children read pending/error via `useFormStatus` or
  `useActionState` without prop drilling.

### Incorrect — manual pending + error state across props

```tsx
function EditNameForm({ user }: { user: User }) {
  const [name, setName] = useState(user.name)
  const [isSaving, setIsSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setIsSaving(true)
    setError(null)
    const err = await updateName(name)
    setIsSaving(false)
    if (err) setError(err)
  }

  return (
    <form onSubmit={handleSubmit}>
      <input value={name} onChange={(e) => setName(e.target.value)} />
      <SubmitButton isSaving={isSaving} />
      {error && <p role="alert">{error}</p>}
    </form>
  )
}

// Child needs the pending flag prop-drilled in.
function SubmitButton({ isSaving }: { isSaving: boolean }) {
  return <button type="submit" disabled={isSaving}>Save</button>
}
```

### Correct — Action + useActionState + useFormStatus

```tsx
function EditNameForm({ user }: { user: User }) {
  const [error, submitAction, isPending] = useActionState(
    async (_prev: string | null, formData: FormData) => {
      const err = await updateName(String(formData.get('name')))
      return err ?? null
    },
    null,
  )

  return (
    <form action={submitAction}>
      <input name="name" defaultValue={user.name} />
      <SubmitButton />
      {!isPending && error && <p role="alert">{error}</p>}
    </form>
  )
}

// Child consumes the form's pending state via context — no prop needed.
function SubmitButton() {
  const { pending } = useFormStatus()
  return <button type="submit" disabled={pending}>Save</button>
}
```

`useActionState` returns `[state, dispatch, isPending]`; the `dispatch`
is what you wire into `action={...}`. `useFormStatus` is a
`react-dom` hook that reads the enclosing form's pending flag, so
descendants stay decoupled from their parent's state.

### When to reach for `useOptimistic`

Pair `useOptimistic` with an Action when the UI can show a provisional
value before the server confirms:

```tsx
const [optimisticCount, addOptimistic] = useOptimistic(count)

async function action(formData: FormData) {
  addOptimistic(optimisticCount + 1)
  await updateCart(formData)
}
```

Keep optimistic updates **pure and reversible** — if the action rejects,
React discards the optimistic value automatically.

### Composition rules of thumb

- The component that **owns** the Action owns the initial state.
- Children needing pending/error read via `useFormStatus` /
  `useActionState`, not props.
- One Action per `<form>`. For multi-button submits, use `formAction`
  on `<button>` to route to different Actions from the same form.

Reference:
- [React 19 — Actions & useActionState](https://react.dev/blog/2024/12/05/react-19)
- [`useFormStatus`](https://react.dev/reference/react-dom/hooks/useFormStatus)
- [`useOptimistic`](https://react.dev/reference/react/useOptimistic)
