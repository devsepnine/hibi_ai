---
title: Optimize List Performance with Stable Object References
impact: CRITICAL
impactDescription: virtualization relies on reference stability
tags: lists, performance, flatlist, virtualization
---

## Optimize List Performance with Stable Object References

표시할 항목을 고르려면 `filter`를 사용해도 된다. 결과 배열은 새로 만들어지지만 각 행의 객체 참조는 유지된다. 변경되지 않은 행까지 매 렌더에서 새 객체로 만들면 메모된 행이 다시 렌더될 수 있다. 행 값이 실제로 바뀌면 그 행을 갱신한다.

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

**검색 입력과 행 데이터를 유지하는 예제**

입력과 각 행이 자신이 사용하는 상태를 구독한다. 검색어가 바뀌면 화면에 표시할 도메인 이름이 갱신된다. 목록에는 저장소에서 읽은 원래 행을 전달한다.

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

필터링이나 정렬로 새 배열을 만들 때도 기존 행 객체를 재사용할 수 있다. 행 자체가 변경되면 해당 행의 새 값을 전달한다.
