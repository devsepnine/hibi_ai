---
title: Use defer or async on Script Tags
impact: HIGH
impactDescription: eliminates render-blocking
tags: rendering, script, defer, async, performance
---

## Script 태그에 defer 또는 async를 사용한다

**Impact: HIGH (렌더 블로킹을 제거)**

`defer`나 `async` 없이 사용된 script 태그는 스크립트를 다운로드하고 실행하는 동안 HTML 파싱을 차단한다. 이는 First Contentful Paint와 Time to Interactive를 지연시킨다.

- **`defer`**: 병렬로 다운로드되며, HTML 파싱 완료 후 실행된다. 실행 순서가 보장된다.
- **`async`**: 병렬로 다운로드되며, 준비되는 즉시 실행된다. 실행 순서는 보장되지 않는다.

DOM이나 다른 스크립트에 의존하는 스크립트에는 `defer`를 사용한다. analytics 같은 독립적인 스크립트에는 `async`를 사용한다.

**잘못된 예 (렌더 블로킹):**

```tsx
export default function Document() {
  return (
    <html>
      <head>
        <script src="https://example.com/analytics.js" />
        <script src="/scripts/utils.js" />
      </head>
      <body>{/* content */}</body>
    </html>
  )
}
```

**올바른 예 (논블로킹):**

```tsx
export default function Document() {
  return (
    <html>
      <head>
        {/* Independent script - use async */}
        <script src="https://example.com/analytics.js" async />
        {/* DOM-dependent script - use defer */}
        <script src="/scripts/utils.js" defer />
      </head>
      <body>{/* content */}</body>
    </html>
  )
}
```

**참고:** Next.js에서는 raw script 태그 대신 `next/script` 컴포넌트와 `strategy` prop을 우선 사용한다.

```tsx
import Script from 'next/script'

export default function Page() {
  return (
    <>
      <Script src="https://example.com/analytics.js" strategy="afterInteractive" />
      <Script src="/scripts/utils.js" strategy="beforeInteractive" />
    </>
  )
}
```

참고: [MDN - Script element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script#defer)
