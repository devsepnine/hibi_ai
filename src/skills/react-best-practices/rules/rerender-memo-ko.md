---
title: Extract to Memoized Components
impact: MEDIUM
impactDescription: enables early returns
tags: rerender, memo, useMemo, optimization
---

## Memoized 컴포넌트로 추출한다

비싼 작업을 memoized 컴포넌트로 추출해 계산 전에 early return이 가능하도록 한다.

**잘못된 예 (loading 중에도 avatar를 계산):**

```tsx
function Profile({ user, loading }: Props) {
  const avatar = useMemo(() => {
    const id = computeAvatarId(user)
    return <Avatar id={id} />
  }, [user])

  if (loading) return <Skeleton />
  return <div>{avatar}</div>
}
```

**올바른 예 (loading 중에 계산을 건너뜀):**

```tsx
const UserAvatar = memo(function UserAvatar({ user }: { user: User }) {
  const id = useMemo(() => computeAvatarId(user), [user])
  return <Avatar id={id} />
})

function Profile({ user, loading }: Props) {
  if (loading) return <Skeleton />
  return (
    <div>
      <UserAvatar user={user} />
    </div>
  )
}
```

**참고:** 프로젝트에 [React Compiler](https://react.dev/learn/react-compiler)가 활성화되어 있다면 `memo()`와 `useMemo()`를 통한 수동 memoization은 불필요하다. 컴파일러가 재렌더를 자동 최적화한다.
