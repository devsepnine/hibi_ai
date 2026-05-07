---
title: Wrap Strings in Text Components
impact: CRITICAL
impactDescription: prevents runtime crash
tags: rendering, text, core
---

## Wrap Strings in Text Components

문자열은 반드시 `<Text>` 안에서 렌더해야 한다. `<View>`의 직접 자식으로
문자열이 들어가면 React Native는 크래시한다.

**Incorrect (crashes):**

```tsx
import { View } from 'react-native'

function Greeting({ name }: { name: string }) {
  return <View>Hello, {name}!</View>
}
// Error: Text strings must be rendered within a <Text> component.
```

**Correct:**

```tsx
import { View, Text } from 'react-native'

function Greeting({ name }: { name: string }) {
  return (
    <View>
      <Text>Hello, {name}!</Text>
    </View>
  )
}
```
