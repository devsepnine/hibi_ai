---
title: Destructure Functions Early in Render (React Compiler)
impact: HIGH
impactDescription: stable references, fewer re-renders
tags: rerender, hooks, performance, react-compiler
---

## Destructure Functions Early in Render

이 규칙은 React Compiler를 사용하는 경우에만 해당한다.

hook에서 가져온 함수는 render 스코프 최상단에서 destructure한다. 객체에 dot
접근으로 함수를 호출하는 일은 절대 하지 않는다. destructure된 함수는 안정적인
참조를 갖지만, dot 접근은 새 참조를 만들어 메모이제이션을 깨뜨린다.

**Incorrect (dotting into object):**

```tsx
import { useRouter } from 'expo-router'

function SaveButton(props) {
  const router = useRouter()

  // bad: react-compiler will key the cache on "props" and "router", which are objects that change each render
  const handlePress = () => {
    props.onSave()
    router.push('/success') // unstable reference
  }

  return <Button onPress={handlePress}>Save</Button>
}
```

**Correct (destructure early):**

```tsx
import { useRouter } from 'expo-router'

function SaveButton({ onSave }) {
  const { push } = useRouter()

  // good: react-compiler will key on push and onSave
  const handlePress = () => {
    onSave()
    push('/success') // stable reference
  }

  return <Button onPress={handlePress}>Save</Button>
}
```
