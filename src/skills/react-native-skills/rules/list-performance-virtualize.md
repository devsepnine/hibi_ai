---
title: Virtualize Long or Growing Lists
impact: HIGH
impactDescription: reduced memory, faster mounts
tags: lists, performance, virtualization, scrollview
---

## Virtualize long or growing lists

Keep a simple layout for a short, bounded settings list that renders comfortably. For a growing feed, rendering every item can increase mount time and memory use. Use the project’s existing virtualized list, such as FlatList or FlashList, to render a window of rows around the viewport.

**Growing feed rendered eagerly**

```tsx
function Feed({ items }: { items: Item[] }) {
  return (
    <ScrollView>
      {items.map((item) => (
        <ItemCard key={item.id} item={item} />
      ))}
    </ScrollView>
  )
}
// 50 items = 50 components mounted, even if only 10 visible
```

**Feed with the built-in FlatList**

```tsx
import { FlatList } from 'react-native'

function Feed({ items }: { items: Item[] }) {
  return (
    <FlatList
      data={items}
      renderItem={({ item }) => <ItemCard item={item} />}
      keyExtractor={(item) => item.id}
    />
  )
}
```

**Alternative, FlashList:**

```tsx
import { FlashList } from '@shopify/flash-list'

function Feed({ items }: { items: Item[] }) {
  return (
    <FlashList
      data={items}
      // if you aren't using React Compiler, wrap these with useCallback
      renderItem={({ item }) => <ItemCard item={item} />}
      keyExtractor={(item) => item.id}
    />
  )
}
```
