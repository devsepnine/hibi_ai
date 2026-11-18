---
title: Use Compressed Images in Lists
impact: HIGH
impactDescription: faster load times, less memory
tags: lists, images, performance, optimization
---

## Use Compressed Images in Lists

Always load compressed, appropriately-sized images in lists. Full-resolution
images consume excessive memory and cause scroll jank. Request thumbnails from
your server or use an image CDN with resize parameters.

**Incorrect, full-resolution images:**

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

**Correct, request appropriately-sized image:**

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

Request a thumbnail sized for the displayed image and the device pixel density. Use the project’s image component and enable caching or placeholders when needed.
