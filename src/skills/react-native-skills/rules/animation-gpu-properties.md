---
title: Animate Transform and Opacity Instead of Layout Properties
impact: HIGH
impactDescription: GPU-accelerated animations, no layout recalculation
tags: animation, performance, reanimated, transform, opacity
---

## Animate Transform and Opacity Instead of Layout Properties

If collapsing a panel must move the rows below it, animate the layout. A scale transform preserves the space the panel occupies. Use transform and opacity for a visual transition within a fixed slot.

**Panel that moves the following rows**

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

**Panel that scales within a fixed slot**

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

On the target device, check sibling positions before and after collapse and measure frame times during the transition.
