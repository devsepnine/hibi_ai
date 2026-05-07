---
title: Use expo-image for Optimized Images
impact: HIGH
impactDescription: memory efficiency, caching, blurhash placeholders, progressive loading
tags: images, performance, expo-image, ui
---

## Use expo-image for Optimized Images

React Native의 `Image` 대신 `expo-image`를 사용한다. 메모리 효율적인 캐싱,
blurhash placeholder, progressive loading을 제공하고 리스트에서 더 좋은
성능을 낸다.

**Incorrect (React Native Image):**

```tsx
import { Image } from 'react-native'

function Avatar({ url }: { url: string }) {
  return <Image source={{ uri: url }} style={styles.avatar} />
}
```

**Correct (expo-image):**

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

cross-platform(web + native)에서는 내부적으로 `expo-image`를 쓰는
`solito/image`의 `SolitoImage`를 사용한다.

Reference: [expo-image](https://docs.expo.dev/versions/latest/sdk/image/)
