---
title: Minimize State Variables and Derive Values
impact: MEDIUM
impactDescription: fewer re-renders, less state drift
tags: state, derived-state, hooks, optimization
---

## Minimize State Variables and Derive Values

state 변수는 가능한 한 적게 둔다. 기존 state나 prop에서 계산할 수 있는
값이라면 state로 저장하지 말고 render 시점에 derive한다. 중복 state는 불필요한
re-render와 동기화 어긋남(state drift)을 만든다.

**Incorrect (redundant state):**

```tsx
function Cart({ items }: { items: Item[] }) {
  const [total, setTotal] = useState(0)
  const [itemCount, setItemCount] = useState(0)

  useEffect(() => {
    setTotal(items.reduce((sum, item) => sum + item.price, 0))
    setItemCount(items.length)
  }, [items])

  return (
    <View>
      <Text>{itemCount} items</Text>
      <Text>Total: ${total}</Text>
    </View>
  )
}
```

**Correct (derived values):**

```tsx
function Cart({ items }: { items: Item[] }) {
  const total = items.reduce((sum, item) => sum + item.price, 0)
  const itemCount = items.length

  return (
    <View>
      <Text>{itemCount} items</Text>
      <Text>Total: ${total}</Text>
    </View>
  )
}
```

**Another example:**

```tsx
// Incorrect: storing both firstName, lastName, AND fullName
const [firstName, setFirstName] = useState('')
const [lastName, setLastName] = useState('')
const [fullName, setFullName] = useState('')

// Correct: derive fullName
const [firstName, setFirstName] = useState('')
const [lastName, setLastName] = useState('')
const fullName = `${firstName} ${lastName}`
```

state는 최소한의 source of truth여야 한다. 그 외 모든 것은 derive한 값이다.

Reference: [Choosing the State Structure](https://react.dev/learn/choosing-the-state-structure)
