---
title: Animate Transform and Opacity Instead of Layout Properties
impact: HIGH
impactDescription: GPU-accelerated animations, no layout recalculation
tags: animation, performance, reanimated, transform, opacity
---

## Animate Transform and Opacity Instead of Layout Properties

패널이 접힐 때 아래 항목도 올라와야 하면 레이아웃을 변경한다. `scaleY`는 패널이 차지하는 자리를 유지한다. 자리는 그대로 두고 패널만 보이거나 사라져야 하면 `transform`과 `opacity`가 적합하다.

**다음 항목이 함께 이동하는 패널**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function CollapsiblePanel({ expanded, children }: { expanded: boolean; children: React.ReactNode }) {
  const animatedStyle = useAnimatedStyle(() => ({
    height: withTiming(expanded ? 200 : 0), // triggers layout on every frame
    overflow: 'hidden',
  }))

  return <Animated.View style={animatedStyle}>{children}</Animated.View>
}
```

**고정된 공간 안에서 축소하는 패널**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function CollapsiblePanel({ expanded, children }: { expanded: boolean; children: React.ReactNode }) {
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

**Correct, animates translateY for slide animations:**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function SlideIn({ visible, children }: { visible: boolean; children: React.ReactNode }) {
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { translateY: withTiming(visible ? 0 : 100) },
    ],
    opacity: withTiming(visible ? 1 : 0),
  }))

  return <Animated.View style={animatedStyle}>{children}</Animated.View>
}
```

대상 기기에서 접기 전후의 주변 요소 위치와 전환 중 프레임 시간을 함께 확인한다.
