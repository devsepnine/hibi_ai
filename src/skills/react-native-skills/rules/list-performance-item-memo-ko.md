---
title: Pass Primitives to List Items for Memoization
impact: HIGH
impactDescription: enables effective memo() comparison
tags: lists, performance, memo, primitives
---

## Pass Primitives to List Items for Memoization

가능하면 리스트 아이템 컴포넌트에 primitive 값(string, number, boolean)만
prop으로 넘긴다. primitive를 쓰면 `memo()`의 shallow comparison이 제대로
작동해서 값이 바뀌지 않은 경우 re-render를 건너뛸 수 있다.

**Incorrect (object prop requires deep comparison):**

```tsx
type User = { id: string; name: string; email: string; avatar: string }

const UserRow = memo(function UserRow({ user }: { user: User }) {
  // memo() compares user by reference, not value
  // If parent creates new user object, this re-renders even if data is same
  return <Text>{user.name}</Text>
})

renderItem={({ item }) => <UserRow user={item} />}
```

이 형태도 최적화가 가능하긴 하지만, 적절히 메모이즈하기가 더 어렵다.

**Correct (primitive props enable shallow comparison):**

```tsx
const UserRow = memo(function UserRow({
  id,
  name,
  email,
}: {
  id: string
  name: string
  email: string
}) {
  // memo() compares each primitive directly
  // Re-renders only if id, name, or email actually changed
  return <Text>{name}</Text>
})

renderItem={({ item }) => (
  <UserRow id={item.id} name={item.name} email={item.email} />
)}
```

**Pass only what you need:**

```tsx
// Incorrect: passing entire item when you only need name
<UserRow user={item} />

// Correct: pass only the fields the component uses
<UserRow name={item.name} avatarUrl={item.avatar} />
```

**For callbacks, hoist or use item ID:**

```tsx
// Incorrect: inline function creates new reference
<UserRow name={item.name} onPress={() => handlePress(item.id)} />

// Correct: pass ID, handle in child
<UserRow id={item.id} name={item.name} />

const UserRow = memo(function UserRow({ id, name }: Props) {
  const handlePress = useCallback(() => {
    // use id here
  }, [id])
  return <Pressable onPress={handlePress}><Text>{name}</Text></Pressable>
})
```

primitive prop은 메모이제이션을 예측 가능하고 효과적으로 만든다.

**Note:** React Compiler를 활성화했다면 `memo()`나 `useCallback()`을 사용할
필요가 없다. 다만 객체 참조에 대한 원칙은 여전히 적용된다.
