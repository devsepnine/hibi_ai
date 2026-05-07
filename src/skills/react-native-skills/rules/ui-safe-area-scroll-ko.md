---
title: Use contentInsetAdjustmentBehavior for Safe Areas
impact: MEDIUM
impactDescription: native safe area handling, no layout shifts
tags: safe-area, scrollview, layout
---

## Use contentInsetAdjustmentBehavior for Safe Areas

콘텐츠를 SafeAreaView로 감싸거나 수동으로 padding을 주는 대신, 루트 ScrollView에
`contentInsetAdjustmentBehavior="automatic"`을 설정한다. 그러면 iOS가 safe area
inset을 native로 처리하면서 스크롤 동작도 적절히 유지된다.

**Incorrect (SafeAreaView wrapper):**

```tsx
import { SafeAreaView, ScrollView, View, Text } from 'react-native'

function MyScreen() {
  return (
    <SafeAreaView style={{ flex: 1 }}>
      <ScrollView>
        <View>
          <Text>Content</Text>
        </View>
      </ScrollView>
    </SafeAreaView>
  )
}
```

**Incorrect (manual safe area padding):**

```tsx
import { ScrollView, View, Text } from 'react-native'
import { useSafeAreaInsets } from 'react-native-safe-area-context'

function MyScreen() {
  const insets = useSafeAreaInsets()

  return (
    <ScrollView contentContainerStyle={{ paddingTop: insets.top }}>
      <View>
        <Text>Content</Text>
      </View>
    </ScrollView>
  )
}
```

**Correct (native content inset adjustment):**

```tsx
import { ScrollView, View, Text } from 'react-native'

function MyScreen() {
  return (
    <ScrollView contentInsetAdjustmentBehavior='automatic'>
      <View>
        <Text>Content</Text>
      </View>
    </ScrollView>
  )
}
```

native 방식은 동적인 safe area(키보드, 툴바)를 다루고, 콘텐츠가 status bar
뒤로 자연스럽게 스크롤되도록 한다.
