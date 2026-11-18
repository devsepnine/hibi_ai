---
title: Use expo-image for Optimized Images
impact: HIGH
impactDescription: memory efficiency, caching, blurhash placeholders, progressive loading
tags: images, performance, expo-image, ui
---

## Choose an image component

Keep the project’s image component when it meets the screen’s needs. React Native Image is sufficient for an avatar with a URL and a fixed size. In a project that uses expo-image, use its placeholder, cache policy, or recycling support where the screen needs those features.

**Avatar using React Native Image**

```tsx
import { Image } from 'react-native'

function Avatar({ url }: { url: string }) {
  return <Image source={{ uri: url }} style={styles.avatar} />
}
```

**Avatar in a project that uses expo-image**

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

- `placeholder`: Blurhash or thumbnail while loading
- `contentFit`: `cover`, `contain`, `fill`, `scale-down`
- `transition`: Fade-in duration in ms
- `priority`: `low`, `normal`, `high`
- `cachePolicy`: `memory`, `disk`, `memory-disk`, `none`
- `recyclingKey`: Unique key for list recycling

Reference: [expo-image](https://docs.expo.dev/versions/latest/sdk/image/)
