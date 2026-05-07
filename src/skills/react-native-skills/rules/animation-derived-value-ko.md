---
title: Prefer useDerivedValue Over useAnimatedReaction
impact: MEDIUM
impactDescription: cleaner code, automatic dependency tracking
tags: animation, reanimated, derived-value
---

## Prefer useDerivedValue Over useAnimatedReaction

shared value를 다른 shared value로부터 derive할 때는 `useAnimatedReaction`이
아니라 `useDerivedValue`를 사용한다. derived value는 선언적이고, 의존성을
자동으로 추적하며, 바로 쓸 수 있는 값을 반환한다. animated reaction은 값을
derive하는 게 아니라 사이드 이펙트를 위한 것이다.

**Incorrect (useAnimatedReaction for derivation):**

```tsx
import { useSharedValue, useAnimatedReaction } from 'react-native-reanimated'

function MyComponent() {
  const progress = useSharedValue(0)
  const opacity = useSharedValue(1)

  useAnimatedReaction(
    () => progress.value,
    (current) => {
      opacity.value = 1 - current
    }
  )

  // ...
}
```

**Correct (useDerivedValue):**

```tsx
import { useSharedValue, useDerivedValue } from 'react-native-reanimated'

function MyComponent() {
  const progress = useSharedValue(0)

  const opacity = useDerivedValue(() => 1 - progress.get())

  // ...
}
```

`useAnimatedReaction`은 값을 만들어내지 않는 사이드 이펙트(예: haptic 트리거,
로깅, `runOnJS` 호출)에만 사용한다.

Reference:
[Reanimated useDerivedValue](https://docs.swmansion.com/react-native-reanimated/docs/core/useDerivedValue)
