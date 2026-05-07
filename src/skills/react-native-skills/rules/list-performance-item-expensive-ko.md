---
title: Keep List Items Lightweight
impact: HIGH
impactDescription: reduces render time for visible items during scroll
tags: lists, performance, virtualization, hooks
---

## Keep List Items Lightweight

리스트 아이템은 가능한 한 저렴하게 렌더해야 한다. hook을 최소화하고, query를
피하며, React Context 접근을 제한한다. 가상화 리스트는 스크롤 동안 많은
아이템을 렌더하므로 비싼 아이템은 jank를 만든다.

**Incorrect (heavy list item):**

```tsx
function ProductRow({ id }: { id: string }) {
  // Bad: query inside list item
  const { data: product } = useQuery(['product', id], () => fetchProduct(id))
  // Bad: multiple context accesses
  const theme = useContext(ThemeContext)
  const user = useContext(UserContext)
  const cart = useContext(CartContext)
  // Bad: expensive computation
  const recommendations = useMemo(
    () => computeRecommendations(product),
    [product]
  )

  return <View>{/* ... */}</View>
}
```

**Correct (lightweight list item):**

```tsx
function ProductRow({ name, price, imageUrl }: Props) {
  // Good: receives only primitives, minimal hooks
  return (
    <View>
      <Image source={{ uri: imageUrl }} />
      <Text>{name}</Text>
      <Text>{price}</Text>
    </View>
  )
}
```

**Move data fetching to parent:**

```tsx
// Parent fetches all data once
function ProductList() {
  const { data: products } = useQuery(['products'], fetchProducts)

  return (
    <LegendList
      data={products}
      renderItem={({ item }) => (
        <ProductRow name={item.name} price={item.price} imageUrl={item.image} />
      )}
    />
  )
}
```

**For shared values, use Zustand selectors instead of Context:**

```tsx
// Incorrect: Context causes re-render when any cart value changes
function ProductRow({ id, name }: Props) {
  const { items } = useContext(CartContext)
  const inCart = items.includes(id)
  // ...
}

// Correct: Zustand selector only re-renders when this specific value changes
function ProductRow({ id, name }: Props) {
  // use Set.has (created once at the root) instead of Array.includes()
  const inCart = useCartStore((s) => s.items.has(id))
  // ...
}
```

**Guidelines for list items:**

- query나 데이터 fetching을 하지 않는다
- 비싼 계산을 하지 않는다 (부모로 옮기거나 부모 레벨에서 메모이즈한다)
- React Context보다 Zustand selector를 선호한다
- useState/useEffect hook을 최소화한다
- pre-computed value를 prop으로 전달한다

목표: 리스트 아이템은 prop을 받아 JSX를 반환하는 단순한 렌더 함수가 되어야
한다.
