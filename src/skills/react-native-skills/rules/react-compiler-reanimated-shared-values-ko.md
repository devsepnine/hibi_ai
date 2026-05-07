---
title: Use .get() and .set() for Reanimated Shared Values (not .value)
impact: LOW
impactDescription: required for React Compiler compatibility
tags: reanimated, react-compiler, shared-values
---

## Use .get() and .set() for Shared Values with React Compiler

React Compiler가 활성화된 상태에서는 Reanimated shared value의 `.value`를
직접 읽거나 쓰지 말고 `.get()`과 `.set()`을 사용한다. compiler는 property
접근을 추적하지 못하므로 명시적 메서드가 올바른 동작을 보장한다.

**Incorrect (breaks with React Compiler):**

```tsx
import { useSharedValue } from 'react-native-reanimated'

function Counter() {
  const count = useSharedValue(0)

  const increment = () => {
    count.value = count.value + 1 // opts out of react compiler
  }

  return <Button onPress={increment} title={`Count: ${count.value}`} />
}
```

**Correct (React Compiler compatible):**

```tsx
import { useSharedValue } from 'react-native-reanimated'

function Counter() {
  const count = useSharedValue(0)

  const increment = () => {
    count.set(count.get() + 1)
  }

  return <Button onPress={increment} title={`Count: ${count.get()}`} />
}
```

자세한 내용은
[Reanimated 문서](https://docs.swmansion.com/react-native-reanimated/docs/core/useSharedValue/#react-compiler-support)를
참고한다.
