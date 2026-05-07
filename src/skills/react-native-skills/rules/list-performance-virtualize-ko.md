---
title: Use a List Virtualizer for Any List
impact: HIGH
impactDescription: reduced memory, faster mounts
tags: lists, performance, virtualization, scrollview
---

## Use a List Virtualizer for Any List

짧은 리스트라도 ScrollView에 children을 map해서 그리지 말고 LegendList나
FlashList 같은 list virtualizer를 사용한다. virtualizer는 visible 아이템만
렌더하므로 메모리 사용량과 mount 시간이 줄어든다. ScrollView는 모든 children을
한 번에 마운트하기 때문에 금방 비싸진다.

**Incorrect (ScrollView renders all items at once):**

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

**Correct (virtualizer renders only visible items):**

```tsx
import { LegendList } from '@legendapp/list'

function Feed({ items }: { items: Item[] }) {
  return (
    <LegendList
      data={items}
      // if you aren't using React Compiler, wrap these with useCallback
      renderItem={({ item }) => <ItemCard item={item} />}
      keyExtractor={(item) => item.id}
      estimatedItemSize={80}
    />
  )
}
// Only ~10-15 visible items mounted at a time
```

**Alternative (FlashList):**

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

이 이점은 스크롤 가능한 콘텐츠가 있는 모든 화면(profile, settings, feed, 검색
결과)에 적용된다. virtualization을 기본값으로 삼는다.
