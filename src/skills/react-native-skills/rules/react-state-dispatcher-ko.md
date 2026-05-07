---
title: useState Dispatch updaters for State That Depends on Current Value
impact: MEDIUM
impactDescription: avoids stale closures, prevents unnecessary re-renders
tags: state, hooks, useState, callbacks
---

## Use Dispatch Updaters for State That Depends on Current Value

다음 state가 현재 state에 의존한다면, 콜백 안에서 state 변수를 직접 읽지 말고
dispatch updater(`setState(prev => ...)`)를 사용한다. 이렇게 해야 stale
closure를 피하고 항상 최신 값과 비교할 수 있다.

**Incorrect (reads state directly):**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  // size may be stale in this closure
  if (size?.width !== width || size?.height !== height) {
    setSize({ width, height })
  }
}
```

**Correct (dispatch updater):**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  setSize((prev) => {
    if (prev?.width === width && prev?.height === height) return prev
    return { width, height }
  })
}
```

updater에서 이전 값을 그대로 반환하면 re-render를 건너뛴다.

primitive state라면 re-render 전에 값을 비교할 필요가 없다.

**Incorrect (unnecessary comparison for primitive state):**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  setSize((prev) => (prev === width ? prev : width))
}
```

**Correct (sets primitive state directly):**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  setSize(width)
}
```

다만 다음 state가 현재 state에 의존한다면, 그때는 여전히 dispatch updater를
사용해야 한다.

**Incorrect (reads state directly from the callback):**

```tsx
const [count, setCount] = useState(0)

const onTap = () => {
  setCount(count + 1)
}
```

**Correct (dispatch updater):**

```tsx
const [count, setCount] = useState(0)

const onTap = () => {
  setCount((prev) => prev + 1)
}
```
