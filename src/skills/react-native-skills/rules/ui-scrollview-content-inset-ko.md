---
title: Use contentInset for Dynamic ScrollView Spacing
impact: LOW
impactDescription: smoother updates, no layout recalculation
tags: scrollview, layout, contentInset, performance
---

## Use contentInset for Dynamic ScrollView Spacing

ScrollView 위/아래 여백이 동적으로 바뀔 수 있는 경우(키보드, 툴바, 동적
콘텐츠) padding 대신 `contentInset`을 사용한다. `contentInset` 변경은 레이아웃
재계산을 트리거하지 않고 콘텐츠 re-render 없이 스크롤 영역만 조정한다.

**Incorrect (padding causes layout recalculation):**

```tsx
function Feed({ bottomOffset }: { bottomOffset: number }) {
  return (
    <ScrollView contentContainerStyle={{ paddingBottom: bottomOffset }}>
      {children}
    </ScrollView>
  )
}
// Changing bottomOffset triggers full layout recalculation
```

**Correct (contentInset for dynamic spacing):**

```tsx
function Feed({ bottomOffset }: { bottomOffset: number }) {
  return (
    <ScrollView
      contentInset={{ bottom: bottomOffset }}
      scrollIndicatorInsets={{ bottom: bottomOffset }}
    >
      {children}
    </ScrollView>
  )
}
// Changing bottomOffset only adjusts scroll bounds
```

스크롤 인디케이터 위치를 맞추기 위해 `contentInset`과 함께
`scrollIndicatorInsets`를 사용한다. 절대 변하지 않는 정적인 여백이라면 padding을
써도 된다.
