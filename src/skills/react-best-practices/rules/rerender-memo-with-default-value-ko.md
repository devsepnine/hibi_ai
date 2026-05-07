---

title: Extract Default Non-primitive Parameter Value from Memoized Component to Constant
impact: MEDIUM
impactDescription: restores memoization by using a constant for default value
tags: rerender, memo, optimization

---

## Memoized 컴포넌트의 non-primitive 기본값은 상수로 추출한다

memoized 컴포넌트가 배열, 함수, 객체 같은 non-primitive 옵셔널 파라미터의 기본값을 가질 때, 해당 파라미터 없이 컴포넌트를 호출하면 memoization이 깨진다. 이는 매 재렌더마다 새 인스턴스가 생성되어 `memo()`의 strict equality 비교를 통과하지 못하기 때문이다.

이 문제를 해결하려면 기본값을 상수로 추출한다.

**잘못된 예 (`onClick`이 매 재렌더마다 다른 값을 가짐):**

```tsx
const UserAvatar = memo(function UserAvatar({ onClick = () => {} }: { onClick?: () => void }) {
  // ...
})

// Used without optional onClick
<UserAvatar />
```

**올바른 예 (안정적인 기본값):**

```tsx
const NOOP = () => {};

const UserAvatar = memo(function UserAvatar({ onClick = NOOP }: { onClick?: () => void }) {
  // ...
})

// Used without optional onClick
<UserAvatar />
```
