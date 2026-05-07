---
title: Use Compound Components Over Polymorphic Children
impact: MEDIUM
impactDescription: flexible composition, clearer API
tags: design-system, components, composition
---

## Use Compound Components Over Polymorphic Children

text 노드가 아닌 컴포넌트가 string children을 받지 못하도록 만든다. 컴포넌트가
string 자식을 받아야 한다면 그것은 `*Text` 형태의 전용 컴포넌트여야 한다.
button처럼 View(또는 Pressable)와 텍스트가 함께 있는 컴포넌트는 `Button`,
`ButtonText`, `ButtonIcon` 같은 compound component로 만든다.

**Incorrect (polymorphic children):**

```tsx
import { Pressable, Text } from 'react-native'

type ButtonProps = {
  children: string | React.ReactNode
  icon?: React.ReactNode
}

function Button({ children, icon }: ButtonProps) {
  return (
    <Pressable>
      {icon}
      {typeof children === 'string' ? <Text>{children}</Text> : children}
    </Pressable>
  )
}

// Usage is ambiguous
<Button icon={<Icon />}>Save</Button>
<Button><CustomText>Save</CustomText></Button>
```

**Correct (compound components):**

```tsx
import { Pressable, Text } from 'react-native'

function Button({ children }: { children: React.ReactNode }) {
  return <Pressable>{children}</Pressable>
}

function ButtonText({ children }: { children: React.ReactNode }) {
  return <Text>{children}</Text>
}

function ButtonIcon({ children }: { children: React.ReactNode }) {
  return <>{children}</>
}

// Usage is explicit and composable
<Button>
  <ButtonIcon><SaveIcon /></ButtonIcon>
  <ButtonText>Save</ButtonText>
</Button>

<Button>
  <ButtonText>Cancel</ButtonText>
</Button>
```
