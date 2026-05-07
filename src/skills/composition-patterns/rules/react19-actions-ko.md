---
title: Compose Forms Around Actions, Not Local Loading State
impact: MEDIUM
impactDescription: removes prop-drilled pending/error boilerplate; enables progressive enhancement
tags: react19, actions, forms, useActionState, useFormStatus, composition
---

## Compose Forms Around Actions, Not Local Loading State

React 19는 **Actions** — `<form action={fn}>`이나 `<button formAction={fn}>`에 전달되는 비동기 함수 — 를 mutation의 주된 composition 경계로 권장한다. `useActionState`, `useFormStatus`, `useOptimistic`과 결합하면 모든 자손에 끼워 넣었던 수작업 `isLoading` / `error` / `startTransition` 배선을 대체한다.

mutation이 **form 형태**일 때 (사용자가 제출, 서버가 업데이트, UI가 반영) Actions를 사용한다. 실제로는 mutation이 아닌 사이드 이펙트 (모달 열기, 로깅)에는 고전적인 `onClick` 핸들러를 그대로 둔다.

### Benefits over manual state

- **Transitions built-in**: pending 상태가 form 경계에 의해 관리된다.
- **Progressive enhancement**: JS가 비활성화되어도 평범한 `<form action="/path">`가 여전히 작동한다. Actions가 네이티브 form 계약을 재사용하기 때문이다.
- **Composable**: 자식들이 prop drilling 없이 `useFormStatus`나 `useActionState`로 pending/error를 읽는다.

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

`useActionState`는 `[state, dispatch, isPending]`을 반환한다; 이 `dispatch`가 `action={...}`에 연결되는 것이다. `useFormStatus`는 둘러싸는 form의 pending 플래그를 읽는 `react-dom` hook으로, 자손이 부모의 상태에 결합되지 않게 한다.

### When to reach for `useOptimistic`

서버가 확인하기 전에 UI가 잠정적인 값을 보여줄 수 있을 때 Action과 `useOptimistic`을 함께 사용한다:

```tsx
const [optimisticCount, addOptimistic] = useOptimistic(count)

async function action(formData: FormData) {
  addOptimistic(optimisticCount + 1)
  await updateCart(formData)
}
```

optimistic 업데이트는 **순수하고 되돌릴 수 있게** 유지한다 — action이 거부되면 React가 optimistic 값을 자동으로 폐기한다.

### Composition rules of thumb

- Action을 **소유한** 컴포넌트가 초기 상태를 소유한다.
- pending/error가 필요한 자식은 prop이 아니라 `useFormStatus` / `useActionState`로 읽는다.
- `<form>`당 하나의 Action. 다중 버튼 제출의 경우, 같은 form에서 다른 Action으로 라우팅하기 위해 `<button>`에 `formAction`을 사용한다.

Reference:
- [React 19 — Actions & useActionState](https://react.dev/blog/2024/12/05/react-19)
- [`useFormStatus`](https://react.dev/reference/react-dom/hooks/useFormStatus)
- [`useOptimistic`](https://react.dev/reference/react/useOptimistic)
