---
title: Use Galeria for Image Galleries and Lightbox
impact: MEDIUM
impactDescription:
  native shared element transitions, pinch-to-zoom, pan-to-close
tags: images, gallery, lightbox, expo-image, ui
---

## Use Galeria for Image Galleries and Lightbox

탭하면 풀스크린으로 보여주는 lightbox 기능을 가진 이미지 갤러리에는
`@nandorojo/galeria`를 사용한다. native shared element transition과
pinch-to-zoom, double-tap zoom, pan-to-close를 제공한다. `expo-image`를 비롯해
어떤 이미지 컴포넌트와도 같이 쓸 수 있다.

**Incorrect (custom modal implementation):**

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

**Correct (Galeria with expo-image):**

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

Reference: [Galeria](https://github.com/nandorojo/galeria)
