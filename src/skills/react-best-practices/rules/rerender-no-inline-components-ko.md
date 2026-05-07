---
title: Don't Define Components Inside Components
impact: HIGH
impactDescription: prevents remount on every render
tags: rerender, components, remount, performance
---

## 컴포넌트 안에서 컴포넌트를 정의하지 않는다

**Impact: HIGH (매 렌더마다 remount되는 것을 방지)**

다른 컴포넌트 안에 컴포넌트를 정의하면 매 렌더마다 새로운 컴포넌트 타입이 만들어진다. React는 매번 다른 컴포넌트로 인식해 완전히 remount하며, 모든 state와 DOM이 파괴된다.

흔한 이유는 props 전달 없이 부모 변수에 접근하기 위해서이다. 항상 props로 전달한다.

**잘못된 예 (매 렌더마다 remount):**

```tsx
function UserProfile({ user, theme }) {
  // Defined inside to access `theme` - BAD
  const Avatar = () => (
    <img
      src={user.avatarUrl}
      className={theme === 'dark' ? 'avatar-dark' : 'avatar-light'}
    />
  )

  // Defined inside to access `user` - BAD
  const Stats = () => (
    <div>
      <span>{user.followers} followers</span>
      <span>{user.posts} posts</span>
    </div>
  )

  return (
    <div>
      <Avatar />
      <Stats />
    </div>
  )
}
```

`UserProfile`이 렌더될 때마다 `Avatar`와 `Stats`는 새로운 컴포넌트 타입이 된다. React는 기존 인스턴스를 unmount하고 새로 mount하며, 내부 state를 잃고 effect를 다시 실행하며 DOM 노드를 재생성한다.

**올바른 예 (props로 전달):**

```tsx
function Avatar({ src, theme }: { src: string; theme: string }) {
  return (
    <img
      src={src}
      className={theme === 'dark' ? 'avatar-dark' : 'avatar-light'}
    />
  )
}

function Stats({ followers, posts }: { followers: number; posts: number }) {
  return (
    <div>
      <span>{followers} followers</span>
      <span>{posts} posts</span>
    </div>
  )
}

function UserProfile({ user, theme }) {
  return (
    <div>
      <Avatar src={user.avatarUrl} theme={theme} />
      <Stats followers={user.followers} posts={user.posts} />
    </div>
  )
}
```

**이 버그의 증상:**
- 키 입력마다 input 필드가 포커스를 잃는다
- 애니메이션이 예기치 않게 다시 시작된다
- `useEffect`의 cleanup/setup이 부모 렌더마다 실행된다
- 컴포넌트 내부 스크롤 위치가 리셋된다
