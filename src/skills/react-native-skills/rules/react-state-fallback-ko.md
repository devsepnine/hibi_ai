---
title: Use fallback state instead of initialState
impact: MEDIUM
impactDescription: reactive fallbacks without syncing
tags: state, hooks, derived-state, props, initialState
---

## Use fallback state instead of initialState

initial state로 `undefined`를 쓰고, nullish coalescing(`??`)으로 부모 또는
서버 값으로 fallback한다. state는 사용자 의도만을 표현해야 한다 — `undefined`는
"사용자가 아직 선택하지 않음"을 뜻한다. 이렇게 하면 initial render뿐 아니라
소스가 바뀔 때마다 반응하는 fallback이 된다.

**Incorrect (syncs state, loses reactivity):**

```tsx
type Props = { fallbackEnabled: boolean }

function Toggle({ fallbackEnabled }: Props) {
  const [enabled, setEnabled] = useState(defaultEnabled)
  // If fallbackEnabled changes, state is stale
  // State mixes user intent with default value

  return <Switch value={enabled} onValueChange={setEnabled} />
}
```

**Correct (state is user intent, reactive fallback):**

```tsx
type Props = { fallbackEnabled: boolean }

function Toggle({ fallbackEnabled }: Props) {
  const [_enabled, setEnabled] = useState<boolean | undefined>(undefined)
  const enabled = _enabled ?? defaultEnabled
  // undefined = user hasn't touched it, falls back to prop
  // If defaultEnabled changes, component reflects it
  // Once user interacts, their choice persists

  return <Switch value={enabled} onValueChange={setEnabled} />
}
```

**With server data:**

```tsx
function ProfileForm({ data }: { data: User }) {
  const [_theme, setTheme] = useState<string | undefined>(undefined)
  const theme = _theme ?? data.theme
  // Shows server value until user overrides
  // Server refetch updates the fallback automatically

  return <ThemePicker value={theme} onChange={setTheme} />
}
```
