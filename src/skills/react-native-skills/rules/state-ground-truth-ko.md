---
title: State Must Represent Ground Truth
impact: HIGH
impactDescription: cleaner logic, easier debugging, single source of truth
tags: state, derived-state, reanimated, hooks
---

## State Must Represent Ground Truth

state 변수 — React `useState`와 Reanimated shared value 모두 — 는 어떤 것의
실제 상태(예: `pressed`, `progress`, `isOpen`)를 표현해야지, derive된 시각적
값(예: `scale`, `opacity`, `translateY`)을 표현해서는 안 된다. 시각적 값은
state로부터 계산이나 interpolation으로 derive한다.

**Incorrect (storing the visual output):**

```tsx
const scale = useSharedValue(1)

const tap = Gesture.Tap()
  .onBegin(() => {
    scale.set(withTiming(0.95))
  })
  .onFinalize(() => {
    scale.set(withTiming(1))
  })

const animatedStyle = useAnimatedStyle(() => ({
  transform: [{ scale: scale.get() }],
}))
```

**Correct (storing the state, deriving the visual):**

```tsx
const pressed = useSharedValue(0) // 0 = not pressed, 1 = pressed

const tap = Gesture.Tap()
  .onBegin(() => {
    pressed.set(withTiming(1))
  })
  .onFinalize(() => {
    pressed.set(withTiming(0))
  })

const animatedStyle = useAnimatedStyle(() => ({
  transform: [{ scale: interpolate(pressed.get(), [0, 1], [1, 0.95]) }],
}))
```

**Why this matters:**

state 변수는 실제 "상태"를 표현해야 하며, 원하는 최종 결과 그 자체가 아니다.

1. **Single source of truth** — state(`pressed`)는 무슨 일이 일어나는지를
   기술하고, 시각적 값은 그것에서 derive된다
2. **Easier to extend** — opacity, rotation 같은 다른 효과를 추가할 때 동일한
   state로부터 interpolation만 추가하면 된다
3. **Debugging** — `pressed = 1`을 검사하는 게 `scale = 0.95`보다 명확하다
4. **Reusable logic** — 동일한 `pressed` 값이 여러 시각적 속성을 구동할 수
   있다

**Same principle for React state:**

```tsx
// Incorrect: storing derived values
const [isExpanded, setIsExpanded] = useState(false)
const [height, setHeight] = useState(0)

useEffect(() => {
  setHeight(isExpanded ? 200 : 0)
}, [isExpanded])

// Correct: derive from state
const [isExpanded, setIsExpanded] = useState(false)
const height = isExpanded ? 200 : 0
```

state는 최소한의 진실이다. 그 외 모든 것은 derive된 값이다.
