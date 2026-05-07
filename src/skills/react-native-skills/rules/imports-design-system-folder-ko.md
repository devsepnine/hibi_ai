---
title: Import from Design System Folder
impact: LOW
impactDescription: enables global changes and easy refactoring
tags: imports, architecture, design-system
---

## Import from Design System Folder

의존성을 design system 폴더에서 re-export한다. 앱 코드는 패키지에서 직접
import하지 않고 design system 폴더에서 import한다. 이렇게 하면 전역적인
변경과 손쉬운 리팩토링이 가능해진다.

**Incorrect (imports directly from package):**

```tsx
import { View, Text } from 'react-native'
import { Button } from '@ui/button'

function Profile() {
  return (
    <View>
      <Text>Hello</Text>
      <Button>Save</Button>
    </View>
  )
}
```

**Correct (imports from design system):**

```tsx
// components/view.tsx
import { View as RNView } from 'react-native'

// ideal: pick the props you will actually use to control implementation
export function View(
  props: Pick<React.ComponentProps<typeof RNView>, 'style' | 'children'>
) {
  return <RNView {...props} />
}
```

```tsx
// components/text.tsx
export { Text } from 'react-native'
```

```tsx
// components/button.tsx
export { Button } from '@ui/button'
```

```tsx
import { View } from '@/components/view'
import { Text } from '@/components/text'
import { Button } from '@/components/button'

function Profile() {
  return (
    <View>
      <Text>Hello</Text>
      <Button>Save</Button>
    </View>
  )
}
```

처음에는 단순히 re-export로 시작한다. 앱 코드를 바꾸지 않고도 나중에
커스터마이즈할 수 있다.
