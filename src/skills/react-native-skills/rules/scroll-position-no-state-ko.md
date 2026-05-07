---
title: Never Track Scroll Position in useState
impact: HIGH
impactDescription: prevents render thrashing during scroll
tags: scroll, performance, reanimated, useRef
---

## Never Track Scroll Position in useState

스크롤 위치를 `useState`에 저장하지 않는다. scroll 이벤트는 매우 빠르게
발생하므로 state 업데이트가 render thrashing과 프레임 드롭을 만든다. 애니메이션
용도라면 Reanimated shared value를, 비반응적 추적이 필요하면 ref를 사용한다.

**Incorrect (useState causes jank):**

```tsx
import { useState } from 'react'
import {
  ScrollView,
  NativeSyntheticEvent,
  NativeScrollEvent,
} from 'react-native'

function Feed() {
  const [scrollY, setScrollY] = useState(0)

  const onScroll = (e: NativeSyntheticEvent<NativeScrollEvent>) => {
    setScrollY(e.nativeEvent.contentOffset.y) // re-renders on every frame
  }

  return <ScrollView onScroll={onScroll} scrollEventThrottle={16} />
}
```

**Correct (Reanimated for animations):**

```tsx
import Animated, {
  useSharedValue,
  useAnimatedScrollHandler,
} from 'react-native-reanimated'

function Feed() {
  const scrollY = useSharedValue(0)

  const onScroll = useAnimatedScrollHandler({
    onScroll: (e) => {
      scrollY.value = e.contentOffset.y // runs on UI thread, no re-render
    },
  })

  return (
    <Animated.ScrollView
      onScroll={onScroll}
      // higher number has better performance, but it fires less often.
      // unset this if you need higher precision over performance.
      scrollEventThrottle={16}
    />
  )
}
```

**Correct (ref for non-reactive tracking):**

```tsx
import { useRef } from 'react'
import {
  ScrollView,
  NativeSyntheticEvent,
  NativeScrollEvent,
} from 'react-native'

function Feed() {
  const scrollY = useRef(0)

  const onScroll = (e: NativeSyntheticEvent<NativeScrollEvent>) => {
    scrollY.current = e.nativeEvent.contentOffset.y // no re-render
  }

  return <ScrollView onScroll={onScroll} scrollEventThrottle={16} />
}
```
