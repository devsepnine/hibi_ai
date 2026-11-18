---
title: Virtualize Long or Growing Lists
impact: HIGH
impactDescription: reduced memory, faster mounts
tags: lists, performance, virtualization, scrollview
---

## 길거나 계속 늘어나는 목록 가상화

항목 수가 적고 상한이 정해진 설정 화면이 원활히 렌더링되면 기존 레이아웃을 유지한다. 계속 늘어나는 피드의 모든 항목을 함께 렌더링하면 초기 표시 시간과 메모리 사용량이 증가할 수 있다. 프로젝트에서 사용하는 FlatList나 FlashList로 화면 주변에 필요한 행을 렌더링한다.

**모든 행을 처음에 렌더링하는 피드**

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

**내장 FlatList를 사용하는 피드**

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
