---
title: Use Native Navigators for Navigation
impact: HIGH
impactDescription: native performance, platform-appropriate UI
tags: navigation, react-navigation, expo-router, native-stack, tabs
---

## Use Native Navigators for Navigation

JS 기반 대신 항상 native navigator를 사용한다. native navigator는 플랫폼
API(iOS의 UINavigationController, Android의 Fragment)를 사용하므로 더 좋은
성능과 native 동작을 제공한다.

**For stacks:** `@react-navigation/native-stack` 또는 expo-router의 기본
stack(native-stack 기반)을 사용한다. `@react-navigation/stack`은 피한다.

**For tabs:** `react-native-bottom-tabs`(native) 또는 expo-router의 native
tab을 사용한다. native 느낌이 중요하다면 `@react-navigation/bottom-tabs`은
피한다.

### Stack Navigation

**Incorrect (JS stack navigator):**

```tsx
import { createStackNavigator } from '@react-navigation/stack'

const Stack = createStackNavigator()

function App() {
  return (
    <Stack.Navigator>
      <Stack.Screen name='Home' component={HomeScreen} />
      <Stack.Screen name='Details' component={DetailsScreen} />
    </Stack.Navigator>
  )
}
```

**Correct (native stack with react-navigation):**

```tsx
import { createNativeStackNavigator } from '@react-navigation/native-stack'

const Stack = createNativeStackNavigator()

function App() {
  return (
    <Stack.Navigator>
      <Stack.Screen name='Home' component={HomeScreen} />
      <Stack.Screen name='Details' component={DetailsScreen} />
    </Stack.Navigator>
  )
}
```

**Correct (expo-router uses native stack by default):**

```tsx
// app/_layout.tsx
import { Stack } from 'expo-router'

export default function Layout() {
  return <Stack />
}
```

### Tab Navigation

**Incorrect (JS bottom tabs):**

```tsx
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs'

const Tab = createBottomTabNavigator()

function App() {
  return (
    <Tab.Navigator>
      <Tab.Screen name='Home' component={HomeScreen} />
      <Tab.Screen name='Settings' component={SettingsScreen} />
    </Tab.Navigator>
  )
}
```

**Correct (native bottom tabs with react-navigation):**

```tsx
import { createNativeBottomTabNavigator } from '@bottom-tabs/react-navigation'

const Tab = createNativeBottomTabNavigator()

function App() {
  return (
    <Tab.Navigator>
      <Tab.Screen
        name='Home'
        component={HomeScreen}
        options={{
          tabBarIcon: () => ({ sfSymbol: 'house' }),
        }}
      />
      <Tab.Screen
        name='Settings'
        component={SettingsScreen}
        options={{
          tabBarIcon: () => ({ sfSymbol: 'gear' }),
        }}
      />
    </Tab.Navigator>
  )
}
```

**Correct (expo-router native tabs):**

```tsx
// app/(tabs)/_layout.tsx
import { NativeTabs } from 'expo-router/unstable-native-tabs'

export default function TabLayout() {
  return (
    <NativeTabs>
      <NativeTabs.Trigger name='index'>
        <NativeTabs.Trigger.Label>Home</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf='house.fill' md='home' />
      </NativeTabs.Trigger>
      <NativeTabs.Trigger name='settings'>
        <NativeTabs.Trigger.Label>Settings</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf='gear' md='settings' />
      </NativeTabs.Trigger>
    </NativeTabs>
  )
}
```

iOS에서 native tab은 각 탭 화면 root의 첫 번째 `ScrollView`에서 자동으로
`contentInsetAdjustmentBehavior`을 활성화하므로, 반투명 탭 바 뒤에서도 콘텐츠가
올바르게 스크롤된다. 비활성화가 필요하면 trigger에 `disableAutomaticContentInsets`을
사용한다.

### Prefer Native Header Options Over Custom Components

**Incorrect (custom header component):**

```tsx
<Stack.Screen
  name='Profile'
  component={ProfileScreen}
  options={{
    header: () => <CustomHeader title='Profile' />,
  }}
/>
```

**Correct (native header options):**

```tsx
<Stack.Screen
  name='Profile'
  component={ProfileScreen}
  options={{
    title: 'Profile',
    headerLargeTitleEnabled: true,
    headerSearchBarOptions: {
      placeholder: 'Search',
    },
  }}
/>
```

native header는 iOS large title, search bar, blur 효과, safe area를 자동으로
처리한다.

### Why Native Navigators

- **Performance**: native 전환과 제스처가 UI thread에서 실행된다
- **Platform behavior**: iOS large title, Android material design이 자동
  적용된다
- **System integration**: 탭 탭으로 scroll-to-top, PiP 회피, 적절한 safe
  area 처리
- **Accessibility**: 플랫폼 접근성 기능이 자동으로 동작한다

Reference:

- [React Navigation Native Stack](https://reactnavigation.org/docs/native-stack-navigator)
- [React Native Bottom Tabs with React Navigation](https://oss.callstack.com/react-native-bottom-tabs/docs/guides/usage-with-react-navigation)
- [React Native Bottom Tabs with Expo Router](https://oss.callstack.com/react-native-bottom-tabs/docs/guides/usage-with-expo-router)
- [Expo Router Native Tabs](https://docs.expo.dev/router/advanced/native-tabs)
