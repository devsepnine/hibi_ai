---
title: Avoid Inline Objects in renderItem
impact: HIGH
impactDescription: prevents unnecessary re-renders of memoized list items
tags: lists, performance, flatlist, virtualization, memo
---

## Avoid Inline Objects in renderItem

`renderItem` 안에서 prop으로 넘길 새 객체를 만들지 않는다. 인라인 객체는 매
render마다 새 참조를 만들어서 memo를 무력화한다. 대신 `item`의 primitive 값을
직접 넘긴다.

**Incorrect (inline object breaks memoization):**

```tsx
function UserList({ users }: { users: User[] }) {
  return (
    <LegendList
      data={users}
      renderItem={({ item }) => (
        <UserRow
          // Bad: new object on every render
          user={{ id: item.id, name: item.name, avatar: item.avatar }}
        />
      )}
    />
  )
}
```

**Incorrect (inline style object):**

```tsx
renderItem={({ item }) => (
  <UserRow
    name={item.name}
    // Bad: new style object on every render
    style={{ backgroundColor: item.isActive ? 'green' : 'gray' }}
  />
)}
```

**Correct (pass item directly or primitives):**

```tsx
function UserList({ users }: { users: User[] }) {
  return (
    <LegendList
      data={users}
      renderItem={({ item }) => (
        // Good: pass the item directly
        <UserRow user={item} />
      )}
    />
  )
}
```

**Correct (pass primitives, derive inside child):**

```tsx
renderItem={({ item }) => (
  <UserRow
    id={item.id}
    name={item.name}
    isActive={item.isActive}
  />
)}

const UserRow = memo(function UserRow({ id, name, isActive }: Props) {
  // Good: derive style inside memoized component
  const backgroundColor = isActive ? 'green' : 'gray'
  return <View style={[styles.row, { backgroundColor }]}>{/* ... */}</View>
})
```

**Correct (hoist static styles in module scope):**

```tsx
const activeStyle = { backgroundColor: 'green' }
const inactiveStyle = { backgroundColor: 'gray' }

renderItem={({ item }) => (
  <UserRow
    name={item.name}
    // Good: stable references
    style={item.isActive ? activeStyle : inactiveStyle}
  />
)}
```

primitive나 안정적인 참조를 넘기면 실제 값이 바뀌지 않은 경우 `memo()`가
re-render를 건너뛸 수 있다.

**Note:** React Compiler를 활성화한 경우 메모이제이션이 자동으로 처리되므로
이런 수동 최적화의 중요성은 떨어진다.
