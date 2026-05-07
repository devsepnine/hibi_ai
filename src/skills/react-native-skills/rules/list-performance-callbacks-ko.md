---
title: Hoist callbacks to the root of lists
impact: MEDIUM
impactDescription: Fewer re-renders and faster lists
tags: tag1, tag2
---

## List performance callbacks

**Impact: HIGH (Fewer re-renders and faster lists)**

리스트 아이템에 콜백 함수를 전달할 때는 리스트 루트에서 콜백 인스턴스를 한
번만 만든다. 그러면 아이템들이 고유 식별자를 들고 그것을 호출한다.

**Incorrect (creates a new callback on each render):**

```typescript
return (
  <LegendList
    renderItem={({ item }) => {
      // bad: creates a new callback on each render
      const onPress = () => handlePress(item.id)
      return <Item key={item.id} item={item} onPress={onPress} />
    }}
  />
)
```

**Correct (a single function instance passed to each item):**

```typescript
const onPress = useCallback(() => handlePress(item.id), [handlePress, item.id])

return (
  <LegendList
    renderItem={({ item }) => (
      <Item key={item.id} item={item} onPress={onPress} />
    )}
  />
)
```

Reference: [Link to documentation or resource](https://example.com)
