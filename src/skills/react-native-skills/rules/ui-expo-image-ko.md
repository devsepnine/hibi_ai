---
title: Use expo-image for Optimized Images
impact: HIGH
impactDescription: memory efficiency, caching, blurhash placeholders, progressive loading
tags: images, performance, expo-image, ui
---

## 이미지 컴포넌트 선택

프로젝트의 이미지 컴포넌트가 화면 요구를 충족하면 유지한다. URL과 고정 크기만 필요한 아바타는 React Native Image로 표시할 수 있다. expo-image를 사용하는 프로젝트라면 필요한 화면에 placeholder, 캐시 정책, 재사용 지원을 적용한다.

**React Native Image를 사용하는 아바타**

```tsx
import { Image } from 'react-native'

function Avatar({ url }: { url: string }) {
  return <Image source={{ uri: url }} style={styles.avatar} />
}
```

**expo-image를 사용하는 프로젝트의 아바타**

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

- `placeholder`: 로딩 중에 표시할 blurhash 또는 thumbnail
- `contentFit`: `cover`, `contain`, `fill`, `scale-down`
- `transition`: fade-in 지속 시간, 단위는 ms
- `priority`: `low`, `normal`, `high`
- `cachePolicy`: `memory`, `disk`, `memory-disk`, `none`
- `recyclingKey`: 리스트 recycling용 unique key

Reference: [expo-image](https://docs.expo.dev/versions/latest/sdk/image/)
