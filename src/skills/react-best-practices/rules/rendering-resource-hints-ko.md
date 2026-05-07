---
title: Use React DOM Resource Hints
impact: HIGH
impactDescription: reduces load time for critical resources
tags: rendering, preload, preconnect, prefetch, resource-hints
---

## React DOM Resource Hint를 사용한다

**Impact: HIGH (핵심 리소스의 로드 시간을 단축)**

React DOM은 브라우저에 곧 필요할 리소스를 알려주는 API들을 제공한다. 특히 서버 컴포넌트에서 사용하면, 클라이언트가 HTML을 받기 전부터 리소스 로딩을 시작할 수 있어 유용하다.

- **`prefetchDNS(href)`**: 곧 연결할 도메인의 DNS를 미리 해석한다
- **`preconnect(href)`**: 서버에 대한 연결(DNS + TCP + TLS)을 미리 수립한다
- **`preload(href, options)`**: 곧 사용할 리소스(stylesheet, font, script, image)를 미리 가져온다
- **`preloadModule(href)`**: 곧 사용할 ES 모듈을 미리 가져온다
- **`preinit(href, options)`**: stylesheet 또는 script를 가져와 즉시 평가한다
- **`preinitModule(href)`**: ES 모듈을 가져와 즉시 평가한다

**예제 (third-party API에 preconnect):**

```tsx
import { preconnect, prefetchDNS } from 'react-dom'

export default function App() {
  prefetchDNS('https://analytics.example.com')
  preconnect('https://api.example.com')

  return <main>{/* content */}</main>
}
```

**예제 (중요 폰트와 스타일을 preload):**

```tsx
import { preload, preinit } from 'react-dom'

export default function RootLayout({ children }) {
  // Preload font file
  preload('/fonts/inter.woff2', { as: 'font', type: 'font/woff2', crossOrigin: 'anonymous' })

  // Fetch and apply critical stylesheet immediately
  preinit('/styles/critical.css', { as: 'style' })

  return (
    <html>
      <body>{children}</body>
    </html>
  )
}
```

**예제 (코드 스플릿 라우트의 모듈을 preload):**

```tsx
import { preloadModule, preinitModule } from 'react-dom'

function Navigation() {
  const preloadDashboard = () => {
    preloadModule('/dashboard.js', { as: 'script' })
  }

  return (
    <nav>
      <a href="/dashboard" onMouseEnter={preloadDashboard}>
        Dashboard
      </a>
    </nav>
  )
}
```

**언제 어느 것을 쓰는가:**

| API | 사용 시점 |
|-----|----------|
| `prefetchDNS` | 나중에 연결할 third-party 도메인 |
| `preconnect` | 즉시 fetch할 API 또는 CDN |
| `preload` | 현재 페이지에 필요한 핵심 리소스 |
| `preloadModule` | 다음 네비게이션에서 사용 가능성이 높은 JS 모듈 |
| `preinit` | 일찍 실행되어야 하는 stylesheet/script |
| `preinitModule` | 일찍 실행되어야 하는 ES 모듈 |

참고: [React DOM Resource Preloading APIs](https://react.dev/reference/react-dom#resource-preloading-apis)
