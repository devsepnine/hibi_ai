---
title: Optimize List Performance with Stable Object References
impact: CRITICAL
impactDescription: virtualization relies on reference stability
tags: lists, performance, flatlist, virtualization
---

## Optimize List Performance with Stable Object References

가상화 리스트로 데이터를 넘기기 전에 map이나 filter를 하지 않는다. 가상화는
무엇이 바뀌었는지를 객체 참조의 안정성으로 판별한다. 새 참조는 visible item
전체를 다시 렌더하게 만든다. 리스트 부모 레벨에서 빈번한 render 자체를
막아야 한다.

필요하다면 리스트 아이템 안에서 context selector를 사용한다.

**Incorrect (creates new object references on every keystroke):**

```tsx
function DomainSearch() {
  const { keyword, setKeyword } = useKeywordZustandState()
  const { data: tlds } = useTlds()

  // Bad: creates new objects on every render, reparenting the entire list on every keystroke
  const domains = tlds.map((tld) => ({
    domain: `${keyword}.${tld.name}`,
    tld: tld.name,
    price: tld.price,
  }))

  return (
    <>
      <TextInput value={keyword} onChangeText={setKeyword} />
      <LegendList
        data={domains}
        renderItem={({ item }) => <DomainItem item={item} keyword={keyword} />}
      />
    </>
  )
}
```

**Correct (stable references, transform inside items):**

```tsx
const renderItem = ({ item }) => <DomainItem tld={item} />

function DomainSearch() {
  const { data: tlds } = useTlds()

  return (
    <LegendList
      // good: as long as the data is stable, LegendList will not re-render the entire list
      data={tlds}
      renderItem={renderItem}
    />
  )
}

function DomainItem({ tld }: { tld: Tld }) {
  // good: transform within items, and don't pass the dynamic data as a prop
  // good: use a selector function from zustand to receive a stable string back
  const domain = useKeywordZustandState((s) => s.keyword + '.' + tld.name)
  return <Text>{domain}</Text>
}
```

**Updating parent array reference:**

내부 객체 참조가 안정적이라면 새 array 인스턴스를 만드는 것은 괜찮다. 예를
들어 객체 리스트를 정렬하는 경우다.

```tsx
// good: creates a new array instance without mutating the inner objects
// good: parent array reference is unaffected by typing and updating "keyword"
const sortedTlds = tlds.toSorted((a, b) => a.name.localeCompare(b.name))

return <LegendList data={sortedTlds} renderItem={renderItem} />
```

새 array 인스턴스 `sortedTlds`가 만들어져도 내부 객체 참조는 안정적이다.

**With zustand for dynamic data (avoids parent re-renders):**

```tsx
const useSearchStore = create<{ keyword: string }>(() => ({ keyword: '' }))

function DomainSearch() {
  const { data: tlds } = useTlds()

  return (
    <>
      <SearchInput />
      <LegendList
        data={tlds}
        // if you aren't using React Compiler, wrap renderItem with useCallback
        renderItem={({ item }) => <DomainItem tld={item} />}
      />
    </>
  )
}

function DomainItem({ tld }: { tld: Tld }) {
  // Select only what you need—component only re-renders when keyword changes
  const keyword = useSearchStore((s) => s.keyword)
  const domain = `${keyword}.${tld.name}`
  return <Text>{domain}</Text>
}
```

이제 가상화는 입력 시 바뀌지 않은 아이템을 건너뛸 수 있다. 키 입력마다 부모가
아니라 visible 아이템(~20개)만 다시 렌더된다.

**Deriving state within list items based on parent data (avoids parent
re-renders):**

데이터가 부모 state에 따라 조건적으로 결정되는 컴포넌트라면 이 패턴이 더
중요해진다. 예를 들어 어떤 아이템이 즐겨찾기 되어 있는지 확인할 때, 아이템
자신이 state에 접근하면 즐겨찾기 토글이 한 컴포넌트만 다시 렌더되게 만들
수 있다.

```tsx
function DomainItemFavoriteButton({ tld }: { tld: Tld }) {
  const isFavorited = useFavoritesStore((s) => s.favorites.has(tld.id))
  return <TldFavoriteButton isFavorited={isFavorited} />
}
```

참고: React Compiler를 쓰는 경우 리스트 아이템 안에서 React Context 값을 직접
읽어도 된다. 다만 대부분의 경우 Zustand selector보다 약간 느릴 수 있는데,
체감 차이는 미미할 수 있다.
