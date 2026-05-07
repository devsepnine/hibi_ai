---
title: Use Native Modals Over JS-Based Bottom Sheets
impact: HIGH
impactDescription: native performance, gestures, accessibility
tags: modals, bottom-sheet, native, react-navigation
---

## Use Native Modals Over JS-Based Bottom Sheets

JS 기반 bottom sheet 라이브러리 대신 `presentationStyle="formSheet"`을 사용한
native `<Modal>`이나 React Navigation v7의 native form sheet를 사용한다.
native modal은 제스처와 접근성, 더 나은 성능을 기본 제공한다. 저수준
프리미티브에는 native UI에 의존한다.

**Incorrect (JS-based bottom sheet):**

```tsx
import BottomSheet from 'custom-js-bottom-sheet'

function MyScreen() {
  const sheetRef = useRef<BottomSheet>(null)

  return (
    <View style={{ flex: 1 }}>
      <Button onPress={() => sheetRef.current?.expand()} title='Open' />
      <BottomSheet ref={sheetRef} snapPoints={['50%', '90%']}>
        <View>
          <Text>Sheet content</Text>
        </View>
      </BottomSheet>
    </View>
  )
}
```

**Correct (native Modal with formSheet):**

```tsx
import { Modal, View, Text, Button } from 'react-native'

function MyScreen() {
  const [visible, setVisible] = useState(false)

  return (
    <View style={{ flex: 1 }}>
      <Button onPress={() => setVisible(true)} title='Open' />
      <Modal
        visible={visible}
        presentationStyle='formSheet'
        animationType='slide'
        onRequestClose={() => setVisible(false)}
      >
        <View>
          <Text>Sheet content</Text>
        </View>
      </Modal>
    </View>
  )
}
```

**Correct (React Navigation v7 native form sheet):**

```tsx
// In your navigator
<Stack.Screen
  name='Details'
  component={DetailsScreen}
  options={{
    presentation: 'formSheet',
    sheetAllowedDetents: 'fitToContents',
  }}
/>
```

native modal은 swipe-to-dismiss, 적절한 키보드 회피, 접근성을 기본으로 제공한다.
