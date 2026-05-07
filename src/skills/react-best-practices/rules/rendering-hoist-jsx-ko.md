---
title: Hoist Static JSX Elements
impact: LOW
impactDescription: avoids re-creation
tags: rendering, jsx, static, optimization
---

## 정적 JSX 요소를 호이스트한다

정적 JSX는 컴포넌트 외부로 추출해 재생성을 피한다.

**잘못된 예 (렌더할 때마다 요소를 재생성):**

```tsx
function LoadingSkeleton() {
  return <div className="animate-pulse h-20 bg-gray-200" />
}

function Container() {
  return (
    <div>
      {loading && <LoadingSkeleton />}
    </div>
  )
}
```

**올바른 예 (동일한 요소를 재사용):**

```tsx
const loadingSkeleton = (
  <div className="animate-pulse h-20 bg-gray-200" />
)

function Container() {
  return (
    <div>
      {loading && loadingSkeleton}
    </div>
  )
}
```

이는 매 렌더마다 재생성하기에 비싼 크고 정적인 SVG 노드에 특히 유용하다.

**참고:** 프로젝트에 [React Compiler](https://react.dev/learn/react-compiler)가 활성화되어 있다면, 컴파일러가 정적 JSX 요소를 자동으로 호이스트하고 컴포넌트 재렌더를 최적화하므로 수동 호이스트는 불필요하다.
