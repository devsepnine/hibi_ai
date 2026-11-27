---
title: React 19 API Changes
impact: MEDIUM
impactDescription: cleaner component definitions and context usage
tags: react19, refs, context, hooks
---

## React 19 API Changes

> **⚠️ React 19+ only.** React 18 이하라면 이 절은 건너뛴다.

함수 컴포넌트는 `ref`를 prop으로 받고, 조건부 컨텍스트 읽기에는 `use`를 쓸 수 있다.

**래퍼를 쓰는 기존 컴포넌트**

```tsx
const ComposerInput = forwardRef<TextInput, Props>((props, ref) => {
  return <TextInput ref={ref} {...props} />
})
```

**Correct, ref as a regular prop:**

```tsx
function ComposerInput({ ref, ...props }: Props & { ref?: React.Ref<TextInput> }) {
  return <TextInput ref={ref} {...props} />
}
```

클라이언트 컴포넌트의 최상위에서 컨텍스트를 읽을 때는 `useContext`가 유효하다. 조건문 안에서 읽어야 하면 `use`를 쓴다. [React useContext](https://react.dev/reference/react/useContext) · [React use](https://react.dev/reference/react/use)
