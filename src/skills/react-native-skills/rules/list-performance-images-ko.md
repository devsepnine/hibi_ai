---
title: Use Compressed Images in Lists
impact: HIGH
impactDescription: faster load times, less memory
tags: lists, images, performance, optimization
---

## Use Compressed Images in Lists

리스트에서는 항상 압축되고 적절한 크기로 만든 이미지를 로드한다. 풀 해상도
이미지는 메모리를 과도하게 잡아먹고 스크롤 jank를 일으킨다. 서버에서 thumbnail을
요청하거나 resize 파라미터를 지원하는 이미지 CDN을 쓴다.

**Incorrect (full-resolution images):**

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

**Correct (request appropriately-sized image):**

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
`expo-image`, 혹은 내부적으로 `expo-image`를 쓰는 `SolitoImage`. retina
디스플레이용으로 표시 크기의 2배 이미지를 요청한다.
