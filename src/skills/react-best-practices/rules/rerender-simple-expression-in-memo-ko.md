---
title: Do not wrap a simple expression with a primitive result type in useMemo
impact: LOW-MEDIUM
impactDescription: wasted computation on every render
tags: rerender, useMemo, optimization
---

## primitive 결과 타입의 단순 표현식을 useMemo로 감싸지 않는다

표현식이 단순(논리/산술 연산자 몇 개 정도)하고 결과 타입이 primitive(boolean, number, string)일 때는 `useMemo`로 감싸지 않는다.
`useMemo` 호출과 hook 의존성 비교가 표현식 자체보다 더 많은 자원을 소비할 수 있다.

**잘못된 예:**

```tsx
function Header({ user, notifications }: Props) {
  const isLoading = useMemo(() => {
    return user.isLoading || notifications.isLoading
  }, [user.isLoading, notifications.isLoading])

  if (isLoading) return <Skeleton />
  // return some markup
}
```

**올바른 예:**

```tsx
function Header({ user, notifications }: Props) {
  const isLoading = user.isLoading || notifications.isLoading

  if (isLoading) return <Skeleton />
  // return some markup
}
```
