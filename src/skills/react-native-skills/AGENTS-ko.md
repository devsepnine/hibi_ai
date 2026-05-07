# React Native Skills

**Version 1.0.0**  
Engineering  
January 2026

> **Note:**  
> 이 문서는 주로 에이전트와 LLM이 React Native 코드베이스를 유지보수하거나
> 생성, 리팩토링할 때 따르도록 만들어졌다. 사람도 유용하게 쓸 수 있지만,
> AI 보조 워크플로우에서의 자동화와 일관성을 위해 최적화되어 있다.

---

## Abstract

AI 에이전트와 LLM을 위해 설계된 React Native 애플리케이션의 종합 성능 최적화
가이드다. 13개 카테고리에 걸친 35개 이상의 규칙을 다루며, 영향도(impact)에
따라 critical(core rendering, list performance)부터 incremental(fonts,
imports)까지 우선순위가 매겨져 있다. 각 규칙은 상세한 설명, 잘못된 구현과
올바른 구현을 비교하는 실전 예시, 그리고 자동 리팩토링과 코드 생성에 도움이
되는 구체적 영향 지표를 포함한다.

---

## Table of Contents

1. [Core Rendering](#1-core-rendering) — **CRITICAL**
   - 1.1 [Never Use && with Potentially Falsy Values](#11-never-use--with-potentially-falsy-values)
   - 1.2 [Wrap Strings in Text Components](#12-wrap-strings-in-text-components)
2. [List Performance](#2-list-performance) — **HIGH**
   - 2.1 [Avoid Inline Objects in renderItem](#21-avoid-inline-objects-in-renderitem)
   - 2.2 [Hoist callbacks to the root of lists](#22-hoist-callbacks-to-the-root-of-lists)
   - 2.3 [Keep List Items Lightweight](#23-keep-list-items-lightweight)
   - 2.4 [Optimize List Performance with Stable Object References](#24-optimize-list-performance-with-stable-object-references)
   - 2.5 [Pass Primitives to List Items for Memoization](#25-pass-primitives-to-list-items-for-memoization)
   - 2.6 [Use a List Virtualizer for Any List](#26-use-a-list-virtualizer-for-any-list)
   - 2.7 [Use Compressed Images in Lists](#27-use-compressed-images-in-lists)
   - 2.8 [Use Item Types for Heterogeneous Lists](#28-use-item-types-for-heterogeneous-lists)
3. [Animation](#3-animation) — **HIGH**
   - 3.1 [Animate Transform and Opacity Instead of Layout Properties](#31-animate-transform-and-opacity-instead-of-layout-properties)
   - 3.2 [Prefer useDerivedValue Over useAnimatedReaction](#32-prefer-usederivedvalue-over-useanimatedreaction)
   - 3.3 [Use GestureDetector for Animated Press States](#33-use-gesturedetector-for-animated-press-states)
4. [Scroll Performance](#4-scroll-performance) — **HIGH**
   - 4.1 [Never Track Scroll Position in useState](#41-never-track-scroll-position-in-usestate)
5. [Navigation](#5-navigation) — **HIGH**
   - 5.1 [Use Native Navigators for Navigation](#51-use-native-navigators-for-navigation)
6. [React State](#6-react-state) — **MEDIUM**
   - 6.1 [Minimize State Variables and Derive Values](#61-minimize-state-variables-and-derive-values)
   - 6.2 [Use fallback state instead of initialState](#62-use-fallback-state-instead-of-initialstate)
   - 6.3 [useState Dispatch updaters for State That Depends on Current Value](#63-usestate-dispatch-updaters-for-state-that-depends-on-current-value)
7. [State Architecture](#7-state-architecture) — **MEDIUM**
   - 7.1 [State Must Represent Ground Truth](#71-state-must-represent-ground-truth)
8. [React Compiler](#8-react-compiler) — **MEDIUM**
   - 8.1 [Destructure Functions Early in Render (React Compiler)](#81-destructure-functions-early-in-render-react-compiler)
   - 8.2 [Use .get() and .set() for Reanimated Shared Values (not .value)](#82-use-get-and-set-for-reanimated-shared-values-not-value)
9. [User Interface](#9-user-interface) — **MEDIUM**
   - 9.1 [Measuring View Dimensions](#91-measuring-view-dimensions)
   - 9.2 [Modern React Native Styling Patterns](#92-modern-react-native-styling-patterns)
   - 9.3 [Use contentInset for Dynamic ScrollView Spacing](#93-use-contentinset-for-dynamic-scrollview-spacing)
   - 9.4 [Use contentInsetAdjustmentBehavior for Safe Areas](#94-use-contentinsetadjustmentbehavior-for-safe-areas)
   - 9.5 [Use expo-image for Optimized Images](#95-use-expo-image-for-optimized-images)
   - 9.6 [Use Galeria for Image Galleries and Lightbox](#96-use-galeria-for-image-galleries-and-lightbox)
   - 9.7 [Use Native Menus for Dropdowns and Context Menus](#97-use-native-menus-for-dropdowns-and-context-menus)
   - 9.8 [Use Native Modals Over JS-Based Bottom Sheets](#98-use-native-modals-over-js-based-bottom-sheets)
   - 9.9 [Use Pressable Instead of Touchable Components](#99-use-pressable-instead-of-touchable-components)
10. [Design System](#10-design-system) — **MEDIUM**
   - 10.1 [Use Compound Components Over Polymorphic Children](#101-use-compound-components-over-polymorphic-children)
11. [Monorepo](#11-monorepo) — **LOW**
   - 11.1 [Install Native Dependencies in App Directory](#111-install-native-dependencies-in-app-directory)
   - 11.2 [Use Single Dependency Versions Across Monorepo](#112-use-single-dependency-versions-across-monorepo)
12. [Third-Party Dependencies](#12-third-party-dependencies) — **LOW**
   - 12.1 [Import from Design System Folder](#121-import-from-design-system-folder)
13. [JavaScript](#13-javascript) — **LOW**
   - 13.1 [Hoist Intl Formatter Creation](#131-hoist-intl-formatter-creation)
14. [Fonts](#14-fonts) — **LOW**
   - 14.1 [Load fonts natively at build time](#141-load-fonts-natively-at-build-time)

---

## 1. Core Rendering

**Impact: CRITICAL**

React Native의 핵심 렌더링 규칙. 위반 시 런타임 크래시 또는 깨진 UI가 발생한다.

### 1.1 Never Use && with Potentially Falsy Values

**Impact: CRITICAL (prevents production crash)**

`value`가 빈 문자열이나

`0`이 될 수 있을 때는 `{value && <Component />}`를 절대 사용하지 않는다. 이
값들은 falsy지만 JSX가 렌더할 수 있다 — React Native가

`<Text>` 바깥에서 텍스트로 렌더하려다 production에서 hard crash를 일으킨다.

**Incorrect: crashes if count is 0 or name is ""**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  return (
    <View>
      {name && <Text>{name}</Text>}
      {count && <Text>{count} items</Text>}
    </View>
  )
}
// If name="" or count=0, renders the falsy value → crash
```

**Correct: ternary with null**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  return (
    <View>
      {name ? <Text>{name}</Text> : null}
      {count ? <Text>{count} items</Text> : null}
    </View>
  )
}
```

**Correct: explicit boolean coercion**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  return (
    <View>
      {!!name && <Text>{name}</Text>}
      {!!count && <Text>{count} items</Text>}
    </View>
  )
}
```

**Best: early return**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  if (!name) return null

  return (
    <View>
      <Text>{name}</Text>
      {count > 0 ? <Text>{count} items</Text> : null}
    </View>
  )
}
```

early return이 가장 명확하다. 인라인 조건을 쓰는 경우라면 ternary나

명시적인 boolean 검사를 선호한다.

**Lint rule:** [eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react/blob/master/docs/rules/jsx-no-leaked-render.md)의

`react/jsx-no-leaked-render`를 활성화하면

자동으로 잡아낸다.

### 1.2 Wrap Strings in Text Components

**Impact: CRITICAL (prevents runtime crash)**

문자열은 반드시 `<Text>` 안에서 렌더해야 한다. `<View>`의

직접 자식으로 문자열이 들어가면 React Native는 크래시한다.

**Incorrect: crashes**

```tsx
import { View } from 'react-native'

function Greeting({ name }: { name: string }) {
  return <View>Hello, {name}!</View>
}
// Error: Text strings must be rendered within a <Text> component.
```

**Correct:**

```tsx
import { View, Text } from 'react-native'

function Greeting({ name }: { name: string }) {
  return (
    <View>
      <Text>Hello, {name}!</Text>
    </View>
  )
}
```

---

## 2. List Performance

**Impact: HIGH**

가상화 리스트(FlatList, LegendList, FlashList)를 매끄러운 스크롤과 빠른
업데이트를 위해 최적화한다.

### 2.1 Avoid Inline Objects in renderItem

**Impact: HIGH (prevents unnecessary re-renders of memoized list items)**

`renderItem` 안에서 prop으로 넘길 새 객체를 만들지 않는다. 인라인 객체는

매 render마다 새 참조를 만들어서 memo를 무력화한다. 대신 `item`의 primitive

값을 직접 넘긴다.

**Incorrect: inline object breaks memoization**

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

**Incorrect: inline style object**

```tsx
renderItem={({ item }) => (
  <UserRow
    name={item.name}
    // Bad: new style object on every render
    style={{ backgroundColor: item.isActive ? 'green' : 'gray' }}
  />
)}
```

**Correct: pass item directly or primitives**

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

**Correct: pass primitives, derive inside child**

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

**Correct: hoist static styles in module scope**

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

### 2.2 Hoist callbacks to the root of lists

**Impact: MEDIUM (Fewer re-renders and faster lists)**

리스트 아이템에 콜백 함수를 전달할 때는 리스트 루트에서 콜백 인스턴스를

한 번만 만든다. 그러면 아이템들이 고유 식별자를 들고 그것을 호출한다.

**Incorrect: creates a new callback on each render**

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

**Correct: a single function instance passed to each item**

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

Reference: [https://example.com](https://example.com)

### 2.3 Keep List Items Lightweight

**Impact: HIGH (reduces render time for visible items during scroll)**

리스트 아이템은 가능한 한 저렴하게 렌더해야 한다. hook을 최소화하고, query를

피하며, React Context 접근을 제한한다. 가상화 리스트는 스크롤 동안 많은

아이템을 렌더하므로 비싼 아이템은 jank를 만든다.

**Incorrect: heavy list item**

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

**Correct: lightweight list item**

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

### 2.4 Optimize List Performance with Stable Object References

**Impact: CRITICAL (virtualization relies on reference stability)**

가상화 리스트로 데이터를 넘기기 전에 map이나 filter를 하지 않는다. 가상화는

무엇이 바뀌었는지를 객체 참조의 안정성으로 판별한다 — 새 참조는 visible

item 전체를 다시 렌더하게 만든다. 리스트 부모 레벨에서 빈번한 render

자체를 막아야 한다.

필요하다면 리스트 아이템 안에서 context selector를 사용한다.

**Incorrect: creates new object references on every keystroke**

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

**Correct: stable references, transform inside items**

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

```tsx
// good: creates a new array instance without mutating the inner objects
// good: parent array reference is unaffected by typing and updating "keyword"
const sortedTlds = tlds.toSorted((a, b) => a.name.localeCompare(b.name))

return <LegendList data={sortedTlds} renderItem={renderItem} />
```

내부 객체 참조가 안정적이라면 새 array 인스턴스를 만드는 것은 괜찮다.

예를 들어 객체 리스트를 정렬하는 경우다:

새 array 인스턴스 `sortedTlds`가 만들어져도 내부 객체 참조는

안정적이다.

**With zustand for dynamic data: avoids parent re-renders**

```tsx
function DomainItemFavoriteButton({ tld }: { tld: Tld }) {
  const isFavorited = useFavoritesStore((s) => s.favorites.has(tld.id))
  return <TldFavoriteButton isFavorited={isFavorited} />
}
```

이제 가상화는 입력 시 바뀌지 않은 아이템을 건너뛸 수 있다. 키 입력마다

부모가 아니라 visible 아이템(~20개)만 다시 렌더된다.

**Deriving state within list items based on parent data (avoids parent

re-renders):**

데이터가 부모 state에 따라 조건적으로 결정되는 컴포넌트라면 이 패턴이 더

중요해진다. 예를 들어 어떤 아이템이 즐겨찾기 되어 있는지 확인할 때, 아이템

자신이 state에 접근하면 즐겨찾기 토글이 한 컴포넌트만 다시 렌더되게 만들

수 있다.

참고: React Compiler를 쓰는 경우 리스트 아이템 안에서 React Context 값을

직접 읽어도 된다. 다만 대부분의 경우 Zustand selector보다 약간 느릴 수

있는데, 체감 차이는 미미할 수 있다.

### 2.5 Pass Primitives to List Items for Memoization

**Impact: HIGH (enables effective memo() comparison)**

가능하면 리스트 아이템 컴포넌트에 primitive 값(string, number, boolean)만

prop으로 넘긴다. primitive를 쓰면 `memo()`의 shallow comparison이 제대로

작동해서 값이 바뀌지 않은 경우 re-render를 건너뛸 수 있다.

**Incorrect: object prop requires deep comparison**

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

**Correct: primitive props enable shallow comparison**

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

### 2.6 Use a List Virtualizer for Any List

**Impact: HIGH (reduced memory, faster mounts)**

짧은 리스트라도 ScrollView에 children을 map해서 그리지 말고 LegendList나

FlashList 같은 list virtualizer를 사용한다. virtualizer는 visible 아이템만

렌더하므로 메모리 사용량과 mount 시간이 줄어든다. ScrollView는 모든 children을

한 번에 마운트하기 때문에 금방 비싸진다.

**Incorrect: ScrollView renders all items at once**

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

**Correct: virtualizer renders only visible items**

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

**Alternative: FlashList**

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

### 2.7 Use Compressed Images in Lists

**Impact: HIGH (faster load times, less memory)**

리스트에서는 항상 압축되고 적절한 크기로 만든 이미지를 로드한다. 풀 해상도

이미지는 메모리를 과도하게 잡아먹고 스크롤 jank를 일으킨다. 서버에서

thumbnail을 요청하거나 resize 파라미터를 지원하는 이미지 CDN을 쓴다.

**Incorrect: full-resolution images**

```tsx
function ProductItem({ product }: { product: Product }) {
  return (
    <View>
      {/* 4000x3000 image loaded for a 100x100 thumbnail */}
      <Image
        source={{ uri: product.imageUrl }}
        style={{ width: 100, height: 100 }}
      />
      <Text>{product.name}</Text>
    </View>
  )
}
```

**Correct: request appropriately-sized image**

```tsx
function ProductItem({ product }: { product: Product }) {
  // Request a 200x200 image (2x for retina)
  const thumbnailUrl = `${product.imageUrl}?w=200&h=200&fit=cover`

  return (
    <View>
      <Image
        source={{ uri: thumbnailUrl }}
        style={{ width: 100, height: 100 }}
        contentFit='cover'
      />
      <Text>{product.name}</Text>
    </View>
  )
}
```

캐싱과 placeholder를 내장한 최적화 이미지 컴포넌트를 사용한다. 예를 들어

`expo-image`, 혹은 내부적으로 `expo-image`를 쓰는 `SolitoImage`.

retina 디스플레이용으로 표시 크기의 2배 이미지를 요청한다.

### 2.8 Use Item Types for Heterogeneous Lists

**Impact: HIGH (efficient recycling, less layout thrashing)**

리스트에 서로 다른 아이템 레이아웃(message, image, header 등)이 섞여 있다면

각 아이템에 `type` 필드를 두고 리스트에 `getItemType`을 제공한다. 이렇게

하면 아이템들이 별도의 recycling pool에 들어가서 message 컴포넌트가 image

컴포넌트 자리에 재사용되는 일이 없어진다.

[LegendList getItemType](https://legendapp.com/open-source/list/api/props/#getitemtype-v2)

**Incorrect: single component with conditionals**

```tsx
type Item = { id: string; text?: string; imageUrl?: string; isHeader?: boolean }

function ListItem({ item }: { item: Item }) {
  if (item.isHeader) {
    return <HeaderItem title={item.text} />
  }
  if (item.imageUrl) {
    return <ImageItem url={item.imageUrl} />
  }
  return <MessageItem text={item.text} />
}

function Feed({ items }: { items: Item[] }) {
  return (
    <LegendList
      data={items}
      renderItem={({ item }) => <ListItem item={item} />}
      recycleItems
    />
  )
}
```

**Correct: typed items with separate components**

```tsx
type HeaderItem = { id: string; type: 'header'; title: string }
type MessageItem = { id: string; type: 'message'; text: string }
type ImageItem = { id: string; type: 'image'; url: string }
type FeedItem = HeaderItem | MessageItem | ImageItem

function Feed({ items }: { items: FeedItem[] }) {
  return (
    <LegendList
      data={items}
      keyExtractor={(item) => item.id}
      getItemType={(item) => item.type}
      renderItem={({ item }) => {
        switch (item.type) {
          case 'header':
            return <SectionHeader title={item.title} />
          case 'message':
            return <MessageRow text={item.text} />
          case 'image':
            return <ImageRow url={item.url} />
        }
      }}
      recycleItems
    />
  )
}
```

**Why this matters:**

```tsx
<LegendList
  data={items}
  keyExtractor={(item) => item.id}
  getItemType={(item) => item.type}
  getEstimatedItemSize={(index, item, itemType) => {
    switch (itemType) {
      case 'header':
        return 48
      case 'message':
        return 72
      case 'image':
        return 300
      default:
        return 72
    }
  }}
  renderItem={({ item }) => {
    /* ... */
  }}
  recycleItems
/>
```

- **Recycling efficiency**: 같은 타입의 아이템이 recycling pool을 공유한다

- **No layout thrashing**: header가 image cell로 재사용되지 않는다

- **Type safety**: TypeScript가 각 분기에서 아이템 타입을 좁힐 수 있다

- **Better size estimation**: `getEstimatedItemSize`를 `itemType`과 함께 써서

  타입별로 정확한 추정값을 줄 수 있다

---

## 3. Animation

**Impact: HIGH**

GPU 가속 애니메이션, Reanimated 패턴, 제스처 중 렌더 thrashing 회피.

### 3.1 Animate Transform and Opacity Instead of Layout Properties

**Impact: HIGH (GPU-accelerated animations, no layout recalculation)**

`width`, `height`, `top`, `left`, `margin`, `padding` 애니메이션은 피한다. 이런 속성들은 매 프레임 레이아웃을 다시 계산하게 만든다. 대신 `transform` (scale, translate)과 `opacity`를 사용한다. 이들은 레이아웃을 트리거하지 않고 GPU 위에서 실행된다.

**Incorrect: animates height, triggers layout every frame**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function CollapsiblePanel({ expanded }: { expanded: boolean }) {
  const animatedStyle = useAnimatedStyle(() => ({
    height: withTiming(expanded ? 200 : 0), // triggers layout on every frame
    overflow: 'hidden',
  }))

  return <Animated.View style={animatedStyle}>{children}</Animated.View>
}
```

**Correct: animates scaleY, GPU-accelerated**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function CollapsiblePanel({ expanded }: { expanded: boolean }) {
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { scaleY: withTiming(expanded ? 1 : 0) },
    ],
    opacity: withTiming(expanded ? 1 : 0),
  }))

  return (
    <Animated.View style={[{ height: 200, transformOrigin: 'top' }, animatedStyle]}>
      {children}
    </Animated.View>
  )
}
```

**Correct: animates translateY for slide animations**

```tsx
import Animated, { useAnimatedStyle, withTiming } from 'react-native-reanimated'

function SlideIn({ visible }: { visible: boolean }) {
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { translateY: withTiming(visible ? 0 : 100) },
    ],
    opacity: withTiming(visible ? 1 : 0),
  }))

  return <Animated.View style={animatedStyle}>{children}</Animated.View>
}
```

GPU 가속 속성: `transform` (translate, scale, rotate), `opacity`. 그 외 모든 속성은 레이아웃을 트리거한다.

### 3.2 Prefer useDerivedValue Over useAnimatedReaction

**Impact: MEDIUM (cleaner code, automatic dependency tracking)**

shared value를 다른 shared value로부터 derive할 때는 `useAnimatedReaction`이

아니라 `useDerivedValue`를 사용한다. derived value는 선언적이고, 의존성을

자동으로 추적하며, 바로 쓸 수 있는 값을 반환한다. animated reaction은 값을

derive하는 게 아니라 사이드 이펙트를 위한 것이다.

[Reanimated useDerivedValue](https://docs.swmansion.com/react-native-reanimated/docs/core/useDerivedValue)

**Incorrect: useAnimatedReaction for derivation**

```tsx
import { useSharedValue, useAnimatedReaction } from 'react-native-reanimated'

function MyComponent() {
  const progress = useSharedValue(0)
  const opacity = useSharedValue(1)

  useAnimatedReaction(
    () => progress.value,
    (current) => {
      opacity.value = 1 - current
    }
  )

  // ...
}
```

**Correct: useDerivedValue**

```tsx
import { useSharedValue, useDerivedValue } from 'react-native-reanimated'

function MyComponent() {
  const progress = useSharedValue(0)

  const opacity = useDerivedValue(() => 1 - progress.get())

  // ...
}
```

`useAnimatedReaction`은 값을 만들어내지 않는 사이드 이펙트(예: haptic 트리거,

로깅, `runOnJS` 호출)에만 사용한다.

### 3.3 Use GestureDetector for Animated Press States

**Impact: MEDIUM (UI thread animations, smoother press feedback)**

press 상태 애니메이션(누르면 scale, opacity 변화)에는 Pressable의

`Gesture.Tap()`과 shared value를 함께 사용해 `GestureDetector`로 처리한다. Pressable의

`onPressIn`/`onPressOut` 대신이다. gesture 콜백은 UI thread에서 worklet으로 실행되므로 press 애니메이션에

JS thread 왕복이 필요 없다.

[Gesture Handler Tap Gesture](https://docs.swmansion.com/react-native-gesture-handler/docs/gestures/tap-gesture)

**Incorrect: Pressable with JS thread callbacks**

```tsx
import { Pressable } from 'react-native'
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  withTiming,
} from 'react-native-reanimated'

function AnimatedButton({ onPress }: { onPress: () => void }) {
  const scale = useSharedValue(1)

  const animatedStyle = useAnimatedStyle(() => ({
    transform: [{ scale: scale.value }],
  }))

  return (
    <Pressable
      onPress={onPress}
      onPressIn={() => (scale.value = withTiming(0.95))}
      onPressOut={() => (scale.value = withTiming(1))}
    >
      <Animated.View style={animatedStyle}>
        <Text>Press me</Text>
      </Animated.View>
    </Pressable>
  )
}
```

**Correct: GestureDetector with UI thread worklets**

```tsx
import { Gesture, GestureDetector } from 'react-native-gesture-handler'
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  withTiming,
  interpolate,
  runOnJS,
} from 'react-native-reanimated'

function AnimatedButton({ onPress }: { onPress: () => void }) {
  // Store the press STATE (0 = not pressed, 1 = pressed)
  const pressed = useSharedValue(0)

  const tap = Gesture.Tap()
    .onBegin(() => {
      pressed.set(withTiming(1))
    })
    .onFinalize(() => {
      pressed.set(withTiming(0))
    })
    .onEnd(() => {
      runOnJS(onPress)()
    })

  // Derive visual values from the state
  const animatedStyle = useAnimatedStyle(() => ({
    transform: [
      { scale: interpolate(withTiming(pressed.get()), [0, 1], [1, 0.95]) },
    ],
  }))

  return (
    <GestureDetector gesture={tap}>
      <Animated.View style={animatedStyle}>
        <Text>Press me</Text>
      </Animated.View>
    </GestureDetector>
  )
}
```

press **상태**(0 또는 1)를 저장한 뒤 `interpolate`로 scale을 derive한다.

이렇게 하면 shared value를 ground truth로 유지할 수 있다. worklet에서 JS

함수를 호출할 때는 `runOnJS`를 사용한다. React Compiler 호환성을 위해

`.set()`과 `.get()`을 쓴다.

---

## 4. Scroll Performance

**Impact: HIGH**

렌더 thrashing 없이 스크롤 위치를 추적한다.

### 4.1 Never Track Scroll Position in useState

**Impact: HIGH (prevents render thrashing during scroll)**

스크롤 위치를 `useState`에 저장하지 않는다. scroll 이벤트는 매우 빠르게 발생하므로

state 업데이트가 render thrashing과 프레임 드롭을 만든다. 애니메이션

용도라면 Reanimated shared value를, 비반응적 추적이 필요하면 ref를 사용한다.

**Incorrect: useState causes jank**

```tsx
import { useState } from 'react'
import {
  ScrollView,
  NativeSyntheticEvent,
  NativeScrollEvent,
} from 'react-native'

function Feed() {
  const [scrollY, setScrollY] = useState(0)

  const onScroll = (e: NativeSyntheticEvent<NativeScrollEvent>) => {
    setScrollY(e.nativeEvent.contentOffset.y) // re-renders on every frame
  }

  return <ScrollView onScroll={onScroll} scrollEventThrottle={16} />
}
```

**Correct: Reanimated for animations**

```tsx
import Animated, {
  useSharedValue,
  useAnimatedScrollHandler,
} from 'react-native-reanimated'

function Feed() {
  const scrollY = useSharedValue(0)

  const onScroll = useAnimatedScrollHandler({
    onScroll: (e) => {
      scrollY.value = e.contentOffset.y // runs on UI thread, no re-render
    },
  })

  return (
    <Animated.ScrollView
      onScroll={onScroll}
      // higher number has better performance, but it fires less often.
      // unset this if you need higher precision over performance.
      scrollEventThrottle={16}
    />
  )
}
```

**Correct: ref for non-reactive tracking**

```tsx
import { useRef } from 'react'
import {
  ScrollView,
  NativeSyntheticEvent,
  NativeScrollEvent,
} from 'react-native'

function Feed() {
  const scrollY = useRef(0)

  const onScroll = (e: NativeSyntheticEvent<NativeScrollEvent>) => {
    scrollY.current = e.nativeEvent.contentOffset.y // no re-render
  }

  return <ScrollView onScroll={onScroll} scrollEventThrottle={16} />
}
```

---

## 5. Navigation

**Impact: HIGH**

stack과 tab navigation에 JS 기반 대안 대신 native navigator 사용.

### 5.1 Use Native Navigators for Navigation

**Impact: HIGH (native performance, platform-appropriate UI)**

JS 기반 대신 항상 native navigator를 사용한다. native navigator는

플랫폼 API(iOS의 UINavigationController, Android의 Fragment)를 사용하므로 더 좋은

성능과 native 동작을 제공한다.

**For stacks:** `@react-navigation/native-stack` 또는 expo-router의 기본

stack(native-stack 기반)을 사용한다. `@react-navigation/stack`은 피한다.

**For tabs:** `react-native-bottom-tabs`(native) 또는 expo-router의 native

tab을 사용한다. native 느낌이 중요하다면 `@react-navigation/bottom-tabs`은 피한다.

- [React Navigation Native Stack](https://reactnavigation.org/docs/native-stack-navigator)

- [React Native Bottom Tabs with React Navigation](https://oss.callstack.com/react-native-bottom-tabs/docs/guides/usage-with-react-navigation)

- [React Native Bottom Tabs with Expo Router](https://oss.callstack.com/react-native-bottom-tabs/docs/guides/usage-with-expo-router)

- [Expo Router Native Tabs](https://docs.expo.dev/router/advanced/native-tabs)

**Incorrect: JS stack navigator**

```tsx
import { createStackNavigator } from '@react-navigation/stack'

const Stack = createStackNavigator()

function App() {
  return (
    <Stack.Navigator>
      <Stack.Screen name='Home' component={HomeScreen} />
      <Stack.Screen name='Details' component={DetailsScreen} />
    </Stack.Navigator>
  )
}
```

**Correct: native stack with react-navigation**

```tsx
import { createNativeStackNavigator } from '@react-navigation/native-stack'

const Stack = createNativeStackNavigator()

function App() {
  return (
    <Stack.Navigator>
      <Stack.Screen name='Home' component={HomeScreen} />
      <Stack.Screen name='Details' component={DetailsScreen} />
    </Stack.Navigator>
  )
}
```

**Correct: expo-router uses native stack by default**

```tsx
// app/_layout.tsx
import { Stack } from 'expo-router'

export default function Layout() {
  return <Stack />
}
```

**Incorrect: JS bottom tabs**

```tsx
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs'

const Tab = createBottomTabNavigator()

function App() {
  return (
    <Tab.Navigator>
      <Tab.Screen name='Home' component={HomeScreen} />
      <Tab.Screen name='Settings' component={SettingsScreen} />
    </Tab.Navigator>
  )
}
```

**Correct: native bottom tabs with react-navigation**

```tsx
import { createNativeBottomTabNavigator } from '@bottom-tabs/react-navigation'

const Tab = createNativeBottomTabNavigator()

function App() {
  return (
    <Tab.Navigator>
      <Tab.Screen
        name='Home'
        component={HomeScreen}
        options={{
          tabBarIcon: () => ({ sfSymbol: 'house' }),
        }}
      />
      <Tab.Screen
        name='Settings'
        component={SettingsScreen}
        options={{
          tabBarIcon: () => ({ sfSymbol: 'gear' }),
        }}
      />
    </Tab.Navigator>
  )
}
```

**Correct: expo-router native tabs**

```tsx
// app/(tabs)/_layout.tsx
import { NativeTabs } from 'expo-router/unstable-native-tabs'

export default function TabLayout() {
  return (
    <NativeTabs>
      <NativeTabs.Trigger name='index'>
        <NativeTabs.Trigger.Label>Home</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf='house.fill' md='home' />
      </NativeTabs.Trigger>
      <NativeTabs.Trigger name='settings'>
        <NativeTabs.Trigger.Label>Settings</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf='gear' md='settings' />
      </NativeTabs.Trigger>
    </NativeTabs>
  )
}
```

iOS에서 native tab은 각 탭 화면 root의 첫 번째 `ScrollView`에서 자동으로

`contentInsetAdjustmentBehavior`을 활성화하므로, 반투명 탭 바 뒤에서도 콘텐츠가

올바르게 스크롤된다. 비활성화가 필요하면 trigger에

`disableAutomaticContentInsets`을 사용한다.

**Incorrect: custom header component**

```tsx
<Stack.Screen
  name='Profile'
  component={ProfileScreen}
  options={{
    header: () => <CustomHeader title='Profile' />,
  }}
/>
```

**Correct: native header options**

```tsx
<Stack.Screen
  name='Profile'
  component={ProfileScreen}
  options={{
    title: 'Profile',
    headerLargeTitleEnabled: true,
    headerSearchBarOptions: {
      placeholder: 'Search',
    },
  }}
/>
```

native header는 iOS large title, search bar, blur 효과, safe area를 자동으로

처리한다.

- **Performance**: native 전환과 제스처가 UI thread에서 실행된다

- **Platform behavior**: iOS large title, Android material design이 자동 적용된다

- **System integration**: 탭 탭으로 scroll-to-top, PiP 회피, 적절한 safe

  area 처리

- **Accessibility**: 플랫폼 접근성 기능이 자동으로 동작한다

---

## 6. React State

**Impact: MEDIUM**

stale closure와 불필요한 re-render를 피하는 React state 관리 패턴.

### 6.1 Minimize State Variables and Derive Values

**Impact: MEDIUM (fewer re-renders, less state drift)**

state 변수는 가능한 한 적게 둔다. 기존 state나 prop에서 계산할 수 있는 값이라면 state로 저장하지 말고 render 시점에 derive한다. 중복 state는 불필요한 re-render와 동기화 어긋남(state drift)을 만든다.

**Incorrect: redundant state**

```tsx
function Cart({ items }: { items: Item[] }) {
  const [total, setTotal] = useState(0)
  const [itemCount, setItemCount] = useState(0)

  useEffect(() => {
    setTotal(items.reduce((sum, item) => sum + item.price, 0))
    setItemCount(items.length)
  }, [items])

  return (
    <View>
      <Text>{itemCount} items</Text>
      <Text>Total: ${total}</Text>
    </View>
  )
}
```

**Correct: derived values**

```tsx
function Cart({ items }: { items: Item[] }) {
  const total = items.reduce((sum, item) => sum + item.price, 0)
  const itemCount = items.length

  return (
    <View>
      <Text>{itemCount} items</Text>
      <Text>Total: ${total}</Text>
    </View>
  )
}
```

**Another example:**

```tsx
// Incorrect: storing both firstName, lastName, AND fullName
const [firstName, setFirstName] = useState('')
const [lastName, setLastName] = useState('')
const [fullName, setFullName] = useState('')

// Correct: derive fullName
const [firstName, setFirstName] = useState('')
const [lastName, setLastName] = useState('')
const fullName = `${firstName} ${lastName}`
```

state는 최소한의 source of truth여야 한다. 그 외 모든 것은 derive한 값이다.

Reference: [https://react.dev/learn/choosing-the-state-structure](https://react.dev/learn/choosing-the-state-structure)

### 6.2 Use fallback state instead of initialState

**Impact: MEDIUM (reactive fallbacks without syncing)**

initial state로 `undefined`를 쓰고, nullish coalescing(`??`)으로

부모 또는 서버 값으로 fallback한다. state는 사용자 의도만을 표현해야 한다 — `undefined`는

"사용자가 아직 선택하지 않음"을 뜻한다. 이렇게 하면 initial render뿐 아니라

소스가 바뀔 때마다 반응하는 fallback이 된다.

**Incorrect: syncs state, loses reactivity**

```tsx
type Props = { fallbackEnabled: boolean }

function Toggle({ fallbackEnabled }: Props) {
  const [enabled, setEnabled] = useState(defaultEnabled)
  // If fallbackEnabled changes, state is stale
  // State mixes user intent with default value

  return <Switch value={enabled} onValueChange={setEnabled} />
}
```

**Correct: state is user intent, reactive fallback**

```tsx
type Props = { fallbackEnabled: boolean }

function Toggle({ fallbackEnabled }: Props) {
  const [_enabled, setEnabled] = useState<boolean | undefined>(undefined)
  const enabled = _enabled ?? defaultEnabled
  // undefined = user hasn't touched it, falls back to prop
  // If defaultEnabled changes, component reflects it
  // Once user interacts, their choice persists

  return <Switch value={enabled} onValueChange={setEnabled} />
}
```

**With server data:**

```tsx
function ProfileForm({ data }: { data: User }) {
  const [_theme, setTheme] = useState<string | undefined>(undefined)
  const theme = _theme ?? data.theme
  // Shows server value until user overrides
  // Server refetch updates the fallback automatically

  return <ThemePicker value={theme} onChange={setTheme} />
}
```

### 6.3 useState Dispatch updaters for State That Depends on Current Value

**Impact: MEDIUM (avoids stale closures, prevents unnecessary re-renders)**

다음 state가 현재 state에 의존한다면, 콜백 안에서 state 변수를 직접 읽지 말고

dispatch updater(`setState(prev => ...)`)를 사용한다. 이렇게 해야 stale

closure를 피하고 항상 최신 값과 비교할 수 있다.

**Incorrect: reads state directly**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  // size may be stale in this closure
  if (size?.width !== width || size?.height !== height) {
    setSize({ width, height })
  }
}
```

**Correct: dispatch updater**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  setSize((prev) => {
    if (prev?.width === width && prev?.height === height) return prev
    return { width, height }
  })
}
```

updater에서 이전 값을 그대로 반환하면 re-render를 건너뛴다.

primitive state라면 re-render 전에 값을 비교할 필요가 없다.

**Incorrect: unnecessary comparison for primitive state**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  setSize((prev) => (prev === width ? prev : width))
}
```

**Correct: sets primitive state directly**

```tsx
const [size, setSize] = useState<Size | undefined>(undefined)

const onLayout = (e: LayoutChangeEvent) => {
  const { width, height } = e.nativeEvent.layout
  setSize(width)
}
```

다만 다음 state가 현재 state에 의존한다면, 그때는 여전히 dispatch updater를

사용해야 한다.

**Incorrect: reads state directly from the callback**

```tsx
const [count, setCount] = useState(0)

const onTap = () => {
  setCount(count + 1)
}
```

**Correct: dispatch updater**

```tsx
const [count, setCount] = useState(0)

const onTap = () => {
  setCount((prev) => prev + 1)
}
```

---

## 7. State Architecture

**Impact: MEDIUM**

state 변수와 derived value에 대한 ground truth 원칙.

### 7.1 State Must Represent Ground Truth

**Impact: HIGH (cleaner logic, easier debugging, single source of truth)**

state 변수 — React `useState`와 Reanimated shared value 모두 — 는

어떤 것의 실제 상태(예: `pressed`, `progress`, `isOpen`)를 표현해야지,

derive된 시각적 값(예: `scale`, `opacity`, `translateY`)을 표현해서는 안 된다. 시각적 값은

state로부터 계산이나 interpolation으로 derive한다.

**Incorrect: storing the visual output**

```tsx
const scale = useSharedValue(1)

const tap = Gesture.Tap()
  .onBegin(() => {
    scale.set(withTiming(0.95))
  })
  .onFinalize(() => {
    scale.set(withTiming(1))
  })

const animatedStyle = useAnimatedStyle(() => ({
  transform: [{ scale: scale.get() }],
}))
```

**Correct: storing the state, deriving the visual**

```tsx
const pressed = useSharedValue(0) // 0 = not pressed, 1 = pressed

const tap = Gesture.Tap()
  .onBegin(() => {
    pressed.set(withTiming(1))
  })
  .onFinalize(() => {
    pressed.set(withTiming(0))
  })

const animatedStyle = useAnimatedStyle(() => ({
  transform: [{ scale: interpolate(pressed.get(), [0, 1], [1, 0.95]) }],
}))
```

**Why this matters:**

state 변수는 실제 "상태"를 표현해야 하며, 원하는 최종 결과 그 자체가 아니다.

1. **Single source of truth** — state(`pressed`)는 무슨 일이 일어나는지를

   기술하고, 시각적 값은 그것에서 derive된다

2. **Easier to extend** — opacity, rotation 같은 다른 효과를 추가할 때 동일한

   state로부터 interpolation만 추가하면 된다

3. **Debugging** — `pressed = 1`을 검사하는 게 `scale = 0.95`보다 명확하다

4. **Reusable logic** — 동일한 `pressed` 값이 여러 시각적 속성을 구동할 수

   있다

**Same principle for React state:**

```tsx
// Incorrect: storing derived values
const [isExpanded, setIsExpanded] = useState(false)
const [height, setHeight] = useState(0)

useEffect(() => {
  setHeight(isExpanded ? 200 : 0)
}, [isExpanded])

// Correct: derive from state
const [isExpanded, setIsExpanded] = useState(false)
const height = isExpanded ? 200 : 0
```

state는 최소한의 진실이다. 그 외 모든 것은 derive된 값이다.

---

## 8. React Compiler

**Impact: MEDIUM**

React Compiler를 React Native와 Reanimated와 함께 쓰기 위한 호환성 패턴.

### 8.1 Destructure Functions Early in Render (React Compiler)

**Impact: HIGH (stable references, fewer re-renders)**

이 규칙은 React Compiler를 사용하는 경우에만 해당한다.

hook에서 가져온 함수는 render 스코프 최상단에서 destructure한다. 객체에 dot

접근으로 함수를 호출하는 일은 절대 하지 않는다. destructure된 함수는 안정적인

참조를 갖지만, dot 접근은 새 참조를 만들어 메모이제이션을 깨뜨린다.

**Incorrect: dotting into object**

```tsx
import { useRouter } from 'expo-router'

function SaveButton(props) {
  const router = useRouter()

  // bad: react-compiler will key the cache on "props" and "router", which are objects that change each render
  const handlePress = () => {
    props.onSave()
    router.push('/success') // unstable reference
  }

  return <Button onPress={handlePress}>Save</Button>
}
```

**Correct: destructure early**

```tsx
import { useRouter } from 'expo-router'

function SaveButton({ onSave }) {
  const { push } = useRouter()

  // good: react-compiler will key on push and onSave
  const handlePress = () => {
    onSave()
    push('/success') // stable reference
  }

  return <Button onPress={handlePress}>Save</Button>
}
```

### 8.2 Use .get() and .set() for Reanimated Shared Values (not .value)

**Impact: LOW (required for React Compiler compatibility)**

React Compiler가 활성화된 상태에서는 Reanimated shared value의

`.value`를 직접 읽거나 쓰지 말고 `.get()`과 `.set()`을 사용한다. compiler는

property 접근을 추적하지 못하므로 명시적 메서드가 올바른 동작을 보장한다.

**Incorrect: breaks with React Compiler**

```tsx
import { useSharedValue } from 'react-native-reanimated'

function Counter() {
  const count = useSharedValue(0)

  const increment = () => {
    count.value = count.value + 1 // opts out of react compiler
  }

  return <Button onPress={increment} title={`Count: ${count.value}`} />
}
```

**Correct: React Compiler compatible**

```tsx
import { useSharedValue } from 'react-native-reanimated'

function Counter() {
  const count = useSharedValue(0)

  const increment = () => {
    count.set(count.get() + 1)
  }

  return <Button onPress={increment} title={`Count: ${count.get()}`} />
}
```

자세한 내용은

[Reanimated 문서](https://docs.swmansion.com/react-native-reanimated/docs/core/useSharedValue/#react-compiler-support)를

참고한다.

---

## 9. User Interface

**Impact: MEDIUM**

이미지, menu, modal, styling, 플랫폼 일관 인터페이스를 위한 네이티브 UI 패턴.

### 9.1 Measuring View Dimensions

**Impact: MEDIUM (synchronous measurement, avoid unnecessary re-renders)**

`useLayoutEffect`(동기 측정)와 `onLayout`(이후 업데이트)을 함께 사용한다.

동기 측정은 마운트 시 즉시 초기 크기를 알려주고, `onLayout`은 view가

바뀌었을 때 값을 최신으로 유지한다. non-primitive state라면 dispatch

updater로 값을 비교해 불필요한 re-render를 막는다.

**Height only:**

```tsx
import { useLayoutEffect, useRef, useState } from 'react'
import { View, LayoutChangeEvent } from 'react-native'

function MeasuredBox({ children }: { children: React.ReactNode }) {
  const ref = useRef<View>(null)
  const [height, setHeight] = useState<number | undefined>(undefined)

  useLayoutEffect(() => {
    // Sync measurement on mount (RN 0.82+)
    const rect = ref.current?.getBoundingClientRect()
    if (rect) setHeight(rect.height)
    // Pre-0.82: ref.current?.measure((x, y, w, h) => setHeight(h))
  }, [])

  const onLayout = (e: LayoutChangeEvent) => {
    setHeight(e.nativeEvent.layout.height)
  }

  return (
    <View ref={ref} onLayout={onLayout}>
      {children}
    </View>
  )
}
```

**Both dimensions:**

```tsx
import { useLayoutEffect, useRef, useState } from 'react'
import { View, LayoutChangeEvent } from 'react-native'

type Size = { width: number; height: number }

function MeasuredBox({ children }: { children: React.ReactNode }) {
  const ref = useRef<View>(null)
  const [size, setSize] = useState<Size | undefined>(undefined)

  useLayoutEffect(() => {
    const rect = ref.current?.getBoundingClientRect()
    if (rect) setSize({ width: rect.width, height: rect.height })
  }, [])

  const onLayout = (e: LayoutChangeEvent) => {
    const { width, height } = e.nativeEvent.layout
    setSize((prev) => {
      // for non-primitive states, compare values before firing a re-render
      if (prev?.width === width && prev?.height === height) return prev
      return { width, height }
    })
  }

  return (
    <View ref={ref} onLayout={onLayout}>
      {children}
    </View>
  )
}
```

함수형 setState를 사용해 비교한다 — 콜백 안에서 state를 직접 읽지 않는다.

### 9.2 Modern React Native Styling Patterns

**Impact: MEDIUM (consistent design, smoother borders, cleaner layouts)**

더 깔끔하고 일관된 React Native 코드를 위해 다음 스타일링 패턴을 따른다.

**`borderRadius`를 쓸 때는 항상 `borderCurve: 'continuous'`도 같이 쓴다:**

**요소 사이 간격에는 margin이 아니라 `gap`을 사용한다:**

```tsx
// Incorrect – margin on children
<View>
  <Text style={{ marginBottom: 8 }}>Title</Text>
  <Text style={{ marginBottom: 8 }}>Subtitle</Text>
</View>

// Correct – gap on parent
<View style={{ gap: 8 }}>
  <Text>Title</Text>
  <Text>Subtitle</Text>
</View>
```

**내부 여백에는 `padding`, 요소 간 간격에는 `gap`을 쓴다:**

```tsx
<View style={{ padding: 16, gap: 12 }}>
  <Text>First</Text>
  <Text>Second</Text>
</View>
```

**linear gradient에는 `experimental_backgroundImage`를 쓴다:**

```tsx
// Incorrect – third-party gradient library
<LinearGradient colors={['#000', '#fff']} />

// Correct – native CSS gradient syntax
<View
  style={{
    experimental_backgroundImage: 'linear-gradient(to bottom, #000, #fff)',
  }}
/>
```

**그림자에는 CSS `boxShadow` 문자열 문법을 사용한다:**

```tsx
// Incorrect – legacy shadow objects or elevation
{ shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.1 }
{ elevation: 4 }

// Correct – CSS box-shadow syntax
{ boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)' }
```

**여러 폰트 사이즈를 사용하지 않고, 강조에는 weight와 color를 활용한다:**

```tsx
// Incorrect – varying font sizes for hierarchy
<Text style={{ fontSize: 18 }}>Title</Text>
<Text style={{ fontSize: 14 }}>Subtitle</Text>
<Text style={{ fontSize: 12 }}>Caption</Text>

// Correct – consistent size, vary weight and color
<Text style={{ fontWeight: '600' }}>Title</Text>
<Text style={{ color: '#666' }}>Subtitle</Text>
<Text style={{ color: '#999' }}>Caption</Text>
```

폰트 사이즈를 제한하면 시각적 일관성이 생긴다. 대신 `fontWeight`(bold/semibold)와

회색조 컬러로 위계를 표현한다.

### 9.3 Use contentInset for Dynamic ScrollView Spacing

**Impact: LOW (smoother updates, no layout recalculation)**

ScrollView 위/아래 여백이 동적으로 바뀔 수 있는 경우(키보드, 툴바, 동적

콘텐츠) padding 대신 `contentInset`을 사용한다. `contentInset` 변경은 레이아웃

재계산을 트리거하지 않고 콘텐츠 re-render 없이 스크롤 영역만 조정한다.

**Incorrect: padding causes layout recalculation**

```tsx
function Feed({ bottomOffset }: { bottomOffset: number }) {
  return (
    <ScrollView contentContainerStyle={{ paddingBottom: bottomOffset }}>
      {children}
    </ScrollView>
  )
}
// Changing bottomOffset triggers full layout recalculation
```

**Correct: contentInset for dynamic spacing**

```tsx
function Feed({ bottomOffset }: { bottomOffset: number }) {
  return (
    <ScrollView
      contentInset={{ bottom: bottomOffset }}
      scrollIndicatorInsets={{ bottom: bottomOffset }}
    >
      {children}
    </ScrollView>
  )
}
// Changing bottomOffset only adjusts scroll bounds
```

스크롤 인디케이터 위치를 맞추기 위해 `contentInset`과 함께

`scrollIndicatorInsets`를 사용한다. 절대 변하지 않는 정적인 여백이라면 padding을

써도 된다.

### 9.4 Use contentInsetAdjustmentBehavior for Safe Areas

**Impact: MEDIUM (native safe area handling, no layout shifts)**

콘텐츠를 SafeAreaView로 감싸거나 수동으로 padding을 주는 대신, 루트 ScrollView에 `contentInsetAdjustmentBehavior="automatic"`을 설정한다. 그러면 iOS가 safe area inset을 native로 처리하면서 스크롤 동작도 적절히 유지된다.

**Incorrect: SafeAreaView wrapper**

```tsx
import { SafeAreaView, ScrollView, View, Text } from 'react-native'

function MyScreen() {
  return (
    <SafeAreaView style={{ flex: 1 }}>
      <ScrollView>
        <View>
          <Text>Content</Text>
        </View>
      </ScrollView>
    </SafeAreaView>
  )
}
```

**Incorrect: manual safe area padding**

```tsx
import { ScrollView, View, Text } from 'react-native'
import { useSafeAreaInsets } from 'react-native-safe-area-context'

function MyScreen() {
  const insets = useSafeAreaInsets()

  return (
    <ScrollView contentContainerStyle={{ paddingTop: insets.top }}>
      <View>
        <Text>Content</Text>
      </View>
    </ScrollView>
  )
}
```

**Correct: native content inset adjustment**

```tsx
import { ScrollView, View, Text } from 'react-native'

function MyScreen() {
  return (
    <ScrollView contentInsetAdjustmentBehavior='automatic'>
      <View>
        <Text>Content</Text>
      </View>
    </ScrollView>
  )
}
```

native 방식은 동적인 safe area(키보드, 툴바)를 다루고, 콘텐츠가 status bar 뒤로 자연스럽게 스크롤되도록 한다.

### 9.5 Use expo-image for Optimized Images

**Impact: HIGH (memory efficiency, caching, blurhash placeholders, progressive loading)**

React Native의 `Image` 대신 `expo-image`를 사용한다. 메모리 효율적인 캐싱, blurhash placeholder, progressive loading을 제공하고 리스트에서 더 좋은 성능을 낸다.

**Incorrect: React Native Image**

```tsx
import { Image } from 'react-native'

function Avatar({ url }: { url: string }) {
  return <Image source={{ uri: url }} style={styles.avatar} />
}
```

**Correct: expo-image**

```tsx
import { Image } from 'expo-image'

function Avatar({ url }: { url: string }) {
  return <Image source={{ uri: url }} style={styles.avatar} />
}
```

**With blurhash placeholder:**

```tsx
<Image
  source={{ uri: url }}
  placeholder={{ blurhash: 'LGF5]+Yk^6#M@-5c,1J5@[or[Q6.' }}
  contentFit="cover"
  transition={200}
  style={styles.image}
/>
```

**With priority and caching:**

```tsx
<Image
  source={{ uri: url }}
  priority="high"
  cachePolicy="memory-disk"
  style={styles.hero}
/>
```

**Key props:**

- `placeholder` — 로딩 중에 표시할 blurhash 또는 thumbnail

- `contentFit` — `cover`, `contain`, `fill`, `scale-down`

- `transition` — fade-in 지속 시간 (ms)

- `priority` — `low`, `normal`, `high`

- `cachePolicy` — `memory`, `disk`, `memory-disk`, `none`

- `recyclingKey` — 리스트 recycling용 unique key

cross-platform(web + native)에서는 내부적으로 `expo-image`를 쓰는 `solito/image`의 `SolitoImage`를 사용한다.

Reference: [https://docs.expo.dev/versions/latest/sdk/image/](https://docs.expo.dev/versions/latest/sdk/image/)

### 9.6 Use Galeria for Image Galleries and Lightbox

**Impact: MEDIUM**

탭하면 풀스크린으로 보여주는 lightbox 기능을 가진 이미지 갤러리에는 `@nandorojo/galeria`를 사용한다.

native shared element transition과 pinch-to-zoom, double-tap

zoom, pan-to-close를 제공한다. `expo-image`를 비롯해 어떤 이미지 컴포넌트와도 같이 쓸 수 있다.

**Incorrect: custom modal implementation**

```tsx
function ImageGallery({ urls }: { urls: string[] }) {
  const [selected, setSelected] = useState<string | null>(null)

  return (
    <>
      {urls.map((url) => (
        <Pressable key={url} onPress={() => setSelected(url)}>
          <Image source={{ uri: url }} style={styles.thumbnail} />
        </Pressable>
      ))}
      <Modal visible={!!selected} onRequestClose={() => setSelected(null)}>
        <Image source={{ uri: selected! }} style={styles.fullscreen} />
      </Modal>
    </>
  )
}
```

**Correct: Galeria with expo-image**

```tsx
import { Galeria } from '@nandorojo/galeria'
import { Image } from 'expo-image'

function ImageGallery({ urls }: { urls: string[] }) {
  return (
    <Galeria urls={urls}>
      {urls.map((url, index) => (
        <Galeria.Image index={index} key={url}>
          <Image source={{ uri: url }} style={styles.thumbnail} />
        </Galeria.Image>
      ))}
    </Galeria>
  )
}
```

**Single image:**

```tsx
import { Galeria } from '@nandorojo/galeria'
import { Image } from 'expo-image'

function Avatar({ url }: { url: string }) {
  return (
    <Galeria urls={[url]}>
      <Galeria.Image>
        <Image source={{ uri: url }} style={styles.avatar} />
      </Galeria.Image>
    </Galeria>
  )
}
```

**With low-res thumbnails and high-res fullscreen:**

```tsx
<Galeria urls={highResUrls}>
  {lowResUrls.map((url, index) => (
    <Galeria.Image index={index} key={url}>
      <Image source={{ uri: url }} style={styles.thumbnail} />
    </Galeria.Image>
  ))}
</Galeria>
```

**With FlashList:**

```tsx
<Galeria urls={urls}>
  <FlashList
    data={urls}
    renderItem={({ item, index }) => (
      <Galeria.Image index={index}>
        <Image source={{ uri: item }} style={styles.thumbnail} />
      </Galeria.Image>
    )}
    numColumns={3}
    estimatedItemSize={100}
  />
</Galeria>
```

`expo-image`, `SolitoImage`, `react-native`의 Image 등 어떤 이미지 컴포넌트와도

함께 동작한다.

Reference: [https://github.com/nandorojo/galeria](https://github.com/nandorojo/galeria)

### 9.7 Use Native Menus for Dropdowns and Context Menus

**Impact: HIGH (native accessibility, platform-consistent UX)**

JS로 만든 커스텀 menu 대신 native 플랫폼 menu를 사용한다. native menu는

접근성을 기본 제공하고, 플랫폼 일관 UX와 더 좋은 성능을 보장한다.

cross-platform native menu에는 [zeego](https://zeego.dev)를 사용한다.

**Incorrect: custom JS menu**

```tsx
import { useState } from 'react'
import { View, Pressable, Text } from 'react-native'

function MyMenu() {
  const [open, setOpen] = useState(false)

  return (
    <View>
      <Pressable onPress={() => setOpen(!open)}>
        <Text>Open Menu</Text>
      </Pressable>
      {open && (
        <View style={{ position: 'absolute', top: 40 }}>
          <Pressable onPress={() => console.log('edit')}>
            <Text>Edit</Text>
          </Pressable>
          <Pressable onPress={() => console.log('delete')}>
            <Text>Delete</Text>
          </Pressable>
        </View>
      )}
    </View>
  )
}
```

**Correct: native menu with zeego**

```tsx
import * as DropdownMenu from 'zeego/dropdown-menu'

function MyMenu() {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <Pressable>
          <Text>Open Menu</Text>
        </Pressable>
      </DropdownMenu.Trigger>

      <DropdownMenu.Content>
        <DropdownMenu.Item key='edit' onSelect={() => console.log('edit')}>
          <DropdownMenu.ItemTitle>Edit</DropdownMenu.ItemTitle>
        </DropdownMenu.Item>

        <DropdownMenu.Item
          key='delete'
          destructive
          onSelect={() => console.log('delete')}
        >
          <DropdownMenu.ItemTitle>Delete</DropdownMenu.ItemTitle>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
```

**Context menu: long-press**

```tsx
import * as ContextMenu from 'zeego/context-menu'

function MyContextMenu() {
  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger>
        <View style={{ padding: 20 }}>
          <Text>Long press me</Text>
        </View>
      </ContextMenu.Trigger>

      <ContextMenu.Content>
        <ContextMenu.Item key='copy' onSelect={() => console.log('copy')}>
          <ContextMenu.ItemTitle>Copy</ContextMenu.ItemTitle>
        </ContextMenu.Item>

        <ContextMenu.Item key='paste' onSelect={() => console.log('paste')}>
          <ContextMenu.ItemTitle>Paste</ContextMenu.ItemTitle>
        </ContextMenu.Item>
      </ContextMenu.Content>
    </ContextMenu.Root>
  )
}
```

**Checkbox items:**

```tsx
import * as DropdownMenu from 'zeego/dropdown-menu'

function SettingsMenu() {
  const [notifications, setNotifications] = useState(true)

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <Pressable>
          <Text>Settings</Text>
        </Pressable>
      </DropdownMenu.Trigger>

      <DropdownMenu.Content>
        <DropdownMenu.CheckboxItem
          key='notifications'
          value={notifications}
          onValueChange={() => setNotifications((prev) => !prev)}
        >
          <DropdownMenu.ItemIndicator />
          <DropdownMenu.ItemTitle>Notifications</DropdownMenu.ItemTitle>
        </DropdownMenu.CheckboxItem>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
```

**Submenus:**

```tsx
import * as DropdownMenu from 'zeego/dropdown-menu'

function MenuWithSubmenu() {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <Pressable>
          <Text>Options</Text>
        </Pressable>
      </DropdownMenu.Trigger>

      <DropdownMenu.Content>
        <DropdownMenu.Item key='home' onSelect={() => console.log('home')}>
          <DropdownMenu.ItemTitle>Home</DropdownMenu.ItemTitle>
        </DropdownMenu.Item>

        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger key='more'>
            <DropdownMenu.ItemTitle>More Options</DropdownMenu.ItemTitle>
          </DropdownMenu.SubTrigger>

          <DropdownMenu.SubContent>
            <DropdownMenu.Item key='settings'>
              <DropdownMenu.ItemTitle>Settings</DropdownMenu.ItemTitle>
            </DropdownMenu.Item>

            <DropdownMenu.Item key='help'>
              <DropdownMenu.ItemTitle>Help</DropdownMenu.ItemTitle>
            </DropdownMenu.Item>
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
```

Reference: [https://zeego.dev/components/dropdown-menu](https://zeego.dev/components/dropdown-menu)

### 9.8 Use Native Modals Over JS-Based Bottom Sheets

**Impact: HIGH (native performance, gestures, accessibility)**

JS 기반 bottom sheet 라이브러리 대신 `presentationStyle="formSheet"`을 사용한

native `<Modal>`이나 React Navigation v7의 native form sheet를 사용한다.

native modal은 제스처와 접근성, 더 나은 성능을 기본 제공한다. 저수준

프리미티브에는 native UI에 의존한다.

**Incorrect: JS-based bottom sheet**

```tsx
import BottomSheet from 'custom-js-bottom-sheet'

function MyScreen() {
  const sheetRef = useRef<BottomSheet>(null)

  return (
    <View style={{ flex: 1 }}>
      <Button onPress={() => sheetRef.current?.expand()} title='Open' />
      <BottomSheet ref={sheetRef} snapPoints={['50%', '90%']}>
        <View>
          <Text>Sheet content</Text>
        </View>
      </BottomSheet>
    </View>
  )
}
```

**Correct: native Modal with formSheet**

```tsx
import { Modal, View, Text, Button } from 'react-native'

function MyScreen() {
  const [visible, setVisible] = useState(false)

  return (
    <View style={{ flex: 1 }}>
      <Button onPress={() => setVisible(true)} title='Open' />
      <Modal
        visible={visible}
        presentationStyle='formSheet'
        animationType='slide'
        onRequestClose={() => setVisible(false)}
      >
        <View>
          <Text>Sheet content</Text>
        </View>
      </Modal>
    </View>
  )
}
```

**Correct: React Navigation v7 native form sheet**

```tsx
// In your navigator
<Stack.Screen
  name='Details'
  component={DetailsScreen}
  options={{
    presentation: 'formSheet',
    sheetAllowedDetents: 'fitToContents',
  }}
/>
```

native modal은 swipe-to-dismiss, 적절한 키보드 회피, 접근성을 기본으로 제공한다.

### 9.9 Use Pressable Instead of Touchable Components

**Impact: LOW (modern API, more flexible)**

`TouchableOpacity`나 `TouchableHighlight`는 사용하지 않는다. 대신

`react-native`나 `react-native-gesture-handler`의 `Pressable`을 사용한다.

**Incorrect: legacy Touchable components**

```tsx
import { TouchableOpacity } from 'react-native'

function MyButton({ onPress }: { onPress: () => void }) {
  return (
    <TouchableOpacity onPress={onPress} activeOpacity={0.7}>
      <Text>Press me</Text>
    </TouchableOpacity>
  )
}
```

**Correct: Pressable**

```tsx
import { Pressable } from 'react-native'

function MyButton({ onPress }: { onPress: () => void }) {
  return (
    <Pressable onPress={onPress}>
      <Text>Press me</Text>
    </Pressable>
  )
}
```

**Correct: Pressable from gesture handler for lists**

```tsx
import { Pressable } from 'react-native-gesture-handler'

function ListItem({ onPress }: { onPress: () => void }) {
  return (
    <Pressable onPress={onPress}>
      <Text>Item</Text>
    </Pressable>
  )
}
```

스크롤되는 리스트 안에서는 제스처 조정 측면에서 더 유리하므로

`react-native-gesture-handler`의 Pressable을 사용한다. 단, ScrollView도

`react-native-gesture-handler` 것을 사용하는 경우에 한한다.

**For animated press states (scale, opacity changes):** Pressable의 style

콜백 대신 Reanimated shared value와 `GestureDetector`를 사용한다.

`animation-gesture-detector-press` 규칙을 참고한다.

---

## 10. Design System

**Impact: MEDIUM**

유지보수 가능한 컴포넌트 라이브러리를 위한 아키텍처 패턴.

### 10.1 Use Compound Components Over Polymorphic Children

**Impact: MEDIUM (flexible composition, clearer API)**

text 노드가 아닌 컴포넌트가 string children을 받지 못하도록 만든다. 컴포넌트가

string 자식을 받아야 한다면 그것은 `*Text` 형태의

전용 컴포넌트여야 한다. button처럼 View(또는

Pressable)와 텍스트가 함께 있는 컴포넌트는 `Button`,

`ButtonText`, `ButtonIcon` 같은 compound component로 만든다.

**Incorrect: polymorphic children**

```tsx
import { Pressable, Text } from 'react-native'

type ButtonProps = {
  children: string | React.ReactNode
  icon?: React.ReactNode
}

function Button({ children, icon }: ButtonProps) {
  return (
    <Pressable>
      {icon}
      {typeof children === 'string' ? <Text>{children}</Text> : children}
    </Pressable>
  )
}

// Usage is ambiguous
<Button icon={<Icon />}>Save</Button>
<Button><CustomText>Save</CustomText></Button>
```

**Correct: compound components**

```tsx
import { Pressable, Text } from 'react-native'

function Button({ children }: { children: React.ReactNode }) {
  return <Pressable>{children}</Pressable>
}

function ButtonText({ children }: { children: React.ReactNode }) {
  return <Text>{children}</Text>
}

function ButtonIcon({ children }: { children: React.ReactNode }) {
  return <>{children}</>
}

// Usage is explicit and composable
<Button>
  <ButtonIcon><SaveIcon /></ButtonIcon>
  <ButtonText>Save</ButtonText>
</Button>

<Button>
  <ButtonText>Cancel</ButtonText>
</Button>
```

---

## 11. Monorepo

**Impact: LOW**

monorepo에서 의존성 관리와 native module 설정.

### 11.1 Install Native Dependencies in App Directory

**Impact: CRITICAL (required for autolinking to work)**

monorepo에서 native code가 포함된 패키지는 native app

디렉토리에 직접 설치해야 한다. autolinking은 앱의 `node_modules`만 스캔하므로 다른 패키지에 설치된 native 의존성은

찾지 못한다.

**Incorrect: native dep in shared package only**

```typescript
packages/
  ui/
    package.json  # has react-native-reanimated
  app/
    package.json  # missing react-native-reanimated
```

autolinking이 실패한다 — native code가 링크되지 않는다.

**Correct: native dep in app directory**

```json
// packages/app/package.json
{
  "dependencies": {
    "react-native-reanimated": "3.16.1"
  }
}
```

shared 패키지가 native 의존성을 사용하더라도, autolinking이 native code를

탐지해 링크할 수 있도록 앱도 그것을 의존성에 명시해야 한다.

### 11.2 Use Single Dependency Versions Across Monorepo

**Impact: MEDIUM (avoids duplicate bundles, version conflicts)**

monorepo 안의 모든 패키지에서 각 의존성의 버전을 단일하게 유지한다.

range보다는 정확한 버전을 선호한다. 여러 버전이 섞이면 번들에 코드가 중복되고,

런타임 충돌과 패키지 간 일관성 없는 동작을 일으킨다.

이를 강제하려면 syncpack 같은 도구를 사용한다. 최후의 수단으로는 yarn resolutions

또는 npm overrides를 쓴다.

**Incorrect: version ranges, multiple versions**

```json
// packages/app/package.json
{
  "dependencies": {
    "react-native-reanimated": "^3.0.0"
  }
}

// packages/ui/package.json
{
  "dependencies": {
    "react-native-reanimated": "^3.5.0"
  }
}
```

**Correct: exact versions, single source of truth**

```json
// package.json (root)
{
  "pnpm": {
    "overrides": {
      "react-native-reanimated": "3.16.1"
    }
  }
}

// packages/app/package.json
{
  "dependencies": {
    "react-native-reanimated": "3.16.1"
  }
}

// packages/ui/package.json
{
  "dependencies": {
    "react-native-reanimated": "3.16.1"
  }
}
```

패키지 매니저의 override/resolution 기능을 root에서 사용해 버전을 강제한다.

의존성을 추가할 때는 `^`나 `~` 없이 정확한 버전을 지정한다.

---

## 12. Third-Party Dependencies

**Impact: LOW**

유지보수성을 위해 서드파티 의존성을 wrapping하고 re-export한다.

### 12.1 Import from Design System Folder

**Impact: LOW (enables global changes and easy refactoring)**

의존성을 design system 폴더에서 re-export한다. 앱 코드는 패키지에서 직접

import하지 않고 design system 폴더에서 import한다. 이렇게 하면 전역적인

변경과 손쉬운 리팩토링이 가능해진다.

**Incorrect: imports directly from package**

```tsx
import { View, Text } from 'react-native'
import { Button } from '@ui/button'

function Profile() {
  return (
    <View>
      <Text>Hello</Text>
      <Button>Save</Button>
    </View>
  )
}
```

**Correct: imports from design system**

```tsx
import { View } from '@/components/view'
import { Text } from '@/components/text'
import { Button } from '@/components/button'

function Profile() {
  return (
    <View>
      <Text>Hello</Text>
      <Button>Save</Button>
    </View>
  )
}
```

처음에는 단순히 re-export로 시작한다. 앱 코드를 바꾸지 않고도 나중에

커스터마이즈할 수 있다.

---

## 13. JavaScript

**Impact: LOW**

비싼 객체 생성의 호이스팅 같은 마이크로 최적화.

### 13.1 Hoist Intl Formatter Creation

**Impact: LOW-MEDIUM (avoids expensive object recreation)**

`Intl.DateTimeFormat`, `Intl.NumberFormat`,

`Intl.RelativeTimeFormat`을 render나 loop 안에서 생성하지 않는다. 이들은

인스턴스화 비용이 크다. locale/options가 정적이라면 모듈 스코프로 호이스팅한다.

**Incorrect: new formatter every render**

```tsx
function Price({ amount }: { amount: number }) {
  const formatter = new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  })
  return <Text>{formatter.format(amount)}</Text>
}
```

**Correct: hoisted to module scope**

```tsx
const currencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
})

function Price({ amount }: { amount: number }) {
  return <Text>{currencyFormatter.format(amount)}</Text>
}
```

**For dynamic locales, memoize:**

```tsx
const dateFormatter = useMemo(
  () => new Intl.DateTimeFormat(locale, { dateStyle: 'medium' }),
  [locale]
)
```

**Common formatters to hoist:**

```tsx
// Module-level formatters
const dateFormatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' })
const timeFormatter = new Intl.DateTimeFormat('en-US', { timeStyle: 'short' })
const percentFormatter = new Intl.NumberFormat('en-US', { style: 'percent' })
const relativeFormatter = new Intl.RelativeTimeFormat('en-US', {
  numeric: 'auto',
})
```

`Intl` 객체 생성은 `RegExp`나 일반 객체보다 훨씬 비싸다. 매 인스턴스화마다

locale 데이터를 파싱하고 내부 lookup table을 만들기 때문이다.

---

## 14. Fonts

**Impact: LOW**

성능 향상을 위한 native font loading.

### 14.1 Load fonts natively at build time

**Impact: LOW (fonts available at launch, no async loading)**

`useFonts`나 `Font.loadAsync` 대신 `expo-font` config plugin을 사용해 빌드

시점에 폰트를 임베드한다. 임베드된 폰트가 더 효율적이다.

[Expo Font Documentation](https://docs.expo.dev/versions/latest/sdk/font/)

**Incorrect: async font loading**

```tsx
import { useFonts } from 'expo-font'
import { Text, View } from 'react-native'

function App() {
  const [fontsLoaded] = useFonts({
    'Geist-Bold': require('./assets/fonts/Geist-Bold.otf'),
  })

  if (!fontsLoaded) {
    return null
  }

  return (
    <View>
      <Text style={{ fontFamily: 'Geist-Bold' }}>Hello</Text>
    </View>
  )
}
```

**Correct: config plugin, fonts embedded at build**

```tsx
import { Text, View } from 'react-native'

function App() {
  // No loading state needed—font is already available
  return (
    <View>
      <Text style={{ fontFamily: 'Geist-Bold' }}>Hello</Text>
    </View>
  )
}
```

config plugin에 폰트를 추가한 뒤에는 `npx expo prebuild`를 실행하고 native

앱을 다시 빌드한다.

---

## References

1. [https://react.dev](https://react.dev)
2. [https://reactnative.dev](https://reactnative.dev)
3. [https://docs.swmansion.com/react-native-reanimated](https://docs.swmansion.com/react-native-reanimated)
4. [https://docs.swmansion.com/react-native-gesture-handler](https://docs.swmansion.com/react-native-gesture-handler)
5. [https://docs.expo.dev](https://docs.expo.dev)
6. [https://legendapp.com/open-source/legend-list](https://legendapp.com/open-source/legend-list)
7. [https://github.com/nandorojo/galeria](https://github.com/nandorojo/galeria)
8. [https://zeego.dev](https://zeego.dev)
