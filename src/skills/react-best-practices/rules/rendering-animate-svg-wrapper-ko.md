---
title: Animate SVG Wrapper Instead of SVG Element
impact: LOW
impactDescription: enables hardware acceleration
tags: rendering, svg, css, animation, performance
---

## SVG 요소가 아닌 래퍼를 애니메이션한다

많은 브라우저는 SVG 요소에 적용된 CSS3 애니메이션에 하드웨어 가속을 지원하지 않는다. SVG를 `<div>`로 감싸고 래퍼를 애니메이션한다.

**잘못된 예 (SVG 직접 애니메이션 - 하드웨어 가속 없음):**

```tsx
function LoadingSpinner() {
  return (
    <svg 
      className="animate-spin"
      width="24" 
      height="24" 
      viewBox="0 0 24 24"
    >
      <circle cx="12" cy="12" r="10" stroke="currentColor" />
    </svg>
  )
}
```

**올바른 예 (래퍼 div 애니메이션 - 하드웨어 가속됨):**

```tsx
function LoadingSpinner() {
  return (
    <div className="animate-spin">
      <svg 
        width="24" 
        height="24" 
        viewBox="0 0 24 24"
      >
        <circle cx="12" cy="12" r="10" stroke="currentColor" />
      </svg>
    </div>
  )
}
```

이 원칙은 모든 CSS transform과 transition (`transform`, `opacity`, `translate`, `scale`, `rotate`)에 적용된다. 래퍼 div를 사용하면 브라우저가 GPU 가속을 활용해 더 부드러운 애니메이션을 제공할 수 있다.
