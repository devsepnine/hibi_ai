---
title: Optimize List Performance with Stable Object References
impact: CRITICAL
impactDescription: virtualization relies on reference stability
tags: lists, performance, flatlist, virtualization
---

## Optimize List Performance with Stable Object References

Use `filter` when the list should display a subset. It creates a new array and preserves the references of retained rows. Cloning unchanged row objects on every render can make memoized rows render again. When a row’s data changes, update that row.

**Incorrect, creates new object references on every keystroke:**

```tsx
function DomainSearch() {
  const { keyword, setKeyword } = useKeywordZustandState()
  const { data: tlds } = useTlds()

  // Bad: creates new objects on every render, changing every item reference on each keystroke
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
        renderItem={({ item }) => <Text>{item.domain}</Text>}
      />
    </>
  )
}
```

**Keep the input and preserve row data**

The input and each row subscribe to the state they use. Editing the keyword updates the displayed domain name. The list receives the original rows from the data source.

```tsx
function SearchInput() {
  const keyword = useKeywordZustandState((state) => state.keyword)
  const setKeyword = useKeywordZustandState((state) => state.setKeyword)
  return <TextInput value={keyword} onChangeText={setKeyword} />
}

function DomainItem({ tld }: { tld: Tld }) {
  const keyword = useKeywordZustandState((state) => state.keyword)
  return <Text>{keyword}.{tld.name}</Text>
}

const renderItem = ({ item }: { item: Tld }) => <DomainItem tld={item} />

function DomainSearch() {
  const { data: tlds } = useTlds()
  return (
    <>
      <SearchInput />
      <LegendList data={tlds} renderItem={renderItem} />
    </>
  )
}
```

A filtered or sorted array can reuse the original row objects. When a row itself changes, pass its new value.
