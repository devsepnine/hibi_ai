---
title: Use GestureDetector for Animated Press States
impact: MEDIUM
impactDescription: UI thread animations, smoother press feedback
tags: animation, gestures, press, reanimated
---

## Use GestureDetector for Animated Press States

press 상태 애니메이션(누르면 scale, opacity 변화)에는 Pressable의
`onPressIn`/`onPressOut` 대신 `GestureDetector`와 `Gesture.Tap()`, shared
value를 함께 사용한다. gesture 콜백은 UI thread에서 worklet으로 실행되므로
press 애니메이션에 JS thread 왕복이 필요 없다.

**Incorrect (Pressable with JS thread callbacks):**

```tsx
import { Pressable } from 'react-native'
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  withTiming,
} from 'react-native-reanimated'

function AnimatedButton({ onPress }: { onPress: () => void }) {
  const scale = useSharedValue(1)

  const animatedStyle = useAnimatedStyle(() => ({
    transform: [{ scale: scale.value }],
  }))

  return (
    <Pressable
      onPress={onPress}
      onPressIn={() => (scale.value = withTiming(0.95))}
      onPressOut={() => (scale.value = withTiming(1))}
    >
      <Animated.View style={animatedStyle}>
        <Text>Press me</Text>
      </Animated.View>
    </Pressable>
  )
}
```

**Correct (GestureDetector with UI thread worklets):**

```tsx
import { Gesture, GestureDetector } from 'react-native-gesture-handler'
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  withTiming,
  interpolate,
  runOnJS,
} from 'react-native-reanimated'

function AnimatedButton({ onPress }: { onPress: () => void }) {
  // Store the press STATE (0 = not pressed, 1 = pressed)
  const pressed = useSharedValue(0)

  const tap = Gesture.Tap()
    .onBegin(() => {
      pressed.set(withTiming(1))
    })
    .onFinalize(() => {
      pressed.set(withTiming(0))
    })
    .onEnd(() => {
      runOnJS(onPress)()
    })

  // Derive visual values from the state
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { scale: interpolate(withTiming(pressed.get()), [0, 1], [1, 0.95]) },
    ],
  }))

  return (
    <GestureDetector gesture={tap}>
      <Animated.View style={animatedStyle}>
        <Text>Press me</Text>
      </Animated.View>
    </GestureDetector>
  )
}
```

press **상태**(0 또는 1)를 저장한 뒤 `interpolate`로 scale을 derive한다.
이렇게 하면 shared value를 ground truth로 유지할 수 있다. worklet에서 JS
함수를 호출할 때는 `runOnJS`를 사용한다. React Compiler 호환성을 위해
`.set()`과 `.get()`을 쓴다.

Reference:
[Gesture Handler Tap Gesture](https://docs.swmansion.com/react-native-gesture-handler/docs/gestures/tap-gesture)
