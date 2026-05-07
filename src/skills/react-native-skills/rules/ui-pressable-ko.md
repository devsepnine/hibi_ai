---
title: Use Pressable Instead of Touchable Components
impact: LOW
impactDescription: modern API, more flexible
tags: ui, pressable, touchable, gestures
---

## Use Pressable Instead of Touchable Components

`TouchableOpacity`나 `TouchableHighlight`는 사용하지 않는다. 대신
`react-native`나 `react-native-gesture-handler`의 `Pressable`을 사용한다.

**Incorrect (legacy Touchable components):**

```tsx
import { TouchableOpacity } from 'react-native'

function MyButton({ onPress }: { onPress: () => void }) {
  return (
    <TouchableOpacity onPress={onPress} activeOpacity={0.7}>
      <Text>Press me</Text>
    </TouchableOpacity>
  )
}
```

**Correct (Pressable):**

```tsx
import { Pressable } from 'react-native'

function MyButton({ onPress }: { onPress: () => void }) {
  return (
    <Pressable onPress={onPress}>
      <Text>Press me</Text>
    </Pressable>
  )
}
```

**Correct (Pressable from gesture handler for lists):**

```tsx
import { Pressable } from 'react-native-gesture-handler'

function ListItem({ onPress }: { onPress: () => void }) {
  return (
    <Pressable onPress={onPress}>
      <Text>Item</Text>
    </Pressable>
  )
}
```

스크롤되는 리스트 안에서는 제스처 조정 측면에서 더 유리하므로
`react-native-gesture-handler`의 Pressable을 사용한다. 단, ScrollView도
`react-native-gesture-handler` 것을 사용하는 경우에 한한다.

**For animated press states (scale, opacity changes):** Pressable의 style
콜백 대신 Reanimated shared value와 `GestureDetector`를 사용한다.
`animation-gesture-detector-press` 규칙을 참고한다.
