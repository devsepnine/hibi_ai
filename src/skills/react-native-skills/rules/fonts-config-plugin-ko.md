---
title: Load fonts natively at build time
impact: LOW
impactDescription: fonts available at launch, no async loading
tags: fonts, expo, performance, config-plugin
---

## Use Expo Config Plugin for Font Loading

`useFonts`나 `Font.loadAsync` 대신 `expo-font` config plugin을 사용해 빌드
시점에 폰트를 임베드한다. 임베드된 폰트가 더 효율적이다.

**Incorrect (async font loading):**

```tsx
import { useFonts } from 'expo-font'
import { Text, View } from 'react-native'

function App() {
  const [fontsLoaded] = useFonts({
    'Geist-Bold': require('./assets/fonts/Geist-Bold.otf'),
  })

  if (!fontsLoaded) {
    return null
  }

  return (
    <View>
      <Text style={{ fontFamily: 'Geist-Bold' }}>Hello</Text>
    </View>
  )
}
```

**Correct (config plugin, fonts embedded at build):**

```json
// app.json
{
  "expo": {
    "plugins": [
      [
        "expo-font",
        {
          "fonts": ["./assets/fonts/Geist-Bold.otf"]
        }
      ]
    ]
  }
}
```

```tsx
import { Text, View } from 'react-native'

function App() {
  // No loading state needed—font is already available
  return (
    <View>
      <Text style={{ fontFamily: 'Geist-Bold' }}>Hello</Text>
    </View>
  )
}
```

config plugin에 폰트를 추가한 뒤에는 `npx expo prebuild`를 실행하고 native
앱을 다시 빌드한다.

Reference:
[Expo Font Documentation](https://docs.expo.dev/versions/latest/sdk/font/)
