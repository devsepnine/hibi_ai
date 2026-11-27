---
title: React 19 API Changes
impact: MEDIUM
impactDescription: cleaner component definitions and context usage
tags: react19, refs, context, hooks
---

## React 19 API Changes

> **⚠️ React 19+ only.** Skip this if you're on React 18 or earlier.

Function components can receive `ref` as a prop and can use `use` to read context conditionally.

**Existing component using a wrapper**

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

Use `useContext` for a top level context read in a client component. Use `use` when the read belongs inside a conditional. [React useContext](https://react.dev/reference/react/useContext) · [React use](https://react.dev/reference/react/use)
