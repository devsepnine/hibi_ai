---
title: Animate Transform and Opacity Instead of Layout Properties
impact: HIGH
impactDescription: GPU-accelerated animations, no layout recalculation
tags: animation, performance, reanimated, transform, opacity
---

## Animate Transform and Opacity Instead of Layout Properties

`width`, `height`, `top`, `left`, `margin`, `padding` 애니메이션은 피한다.
이런 속성들은 매 프레임 레이아웃을 다시 계산하게 만든다. 대신 `transform`
(scale, translate)과 `opacity`를 사용한다. 이들은 레이아웃을 트리거하지 않고
GPU 위에서 실행된다.

**Incorrect (animates height, triggers layout every frame):**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function CollapsiblePanel({ expanded }: { expanded: boolean }) {
  const animatedStyle = useAnimatedStyle(() => ({
    height: withTiming(expanded ? 200 : 0), // triggers layout on every frame
    overflow: 'hidden',
  }))

  return <Animated.View style={animatedStyle}>{children}</Animated.View>
}
```

**Correct (animates scaleY, GPU-accelerated):**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function CollapsiblePanel({ expanded }: { expanded: boolean }) {
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { scaleY: withTiming(expanded ? 1 : 0) },
    ],
    opacity: withTiming(expanded ? 1 : 0),
  }))

  return (
    <Animated.View style={[{ height: 200, transformOrigin: 'top' }, animatedStyle]}>
      {children}
    </Animated.View>
  )
}
```

**Correct (animates translateY for slide animations):**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function SlideIn({ visible }: { visible: boolean }) {
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { translateY: withTiming(visible ? 0 : 100) },
    ],
    opacity: withTiming(visible ? 1 : 0),
  }))

  return <Animated.View style={animatedStyle}>{children}</Animated.View>
}
```

GPU 가속 속성: `transform` (translate, scale, rotate), `opacity`. 그 외 모든
속성은 레이아웃을 트리거한다.
