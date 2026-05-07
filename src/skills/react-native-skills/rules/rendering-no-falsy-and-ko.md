---
title: Never Use && with Potentially Falsy Values
impact: CRITICAL
impactDescription: prevents production crash
tags: rendering, conditional, jsx, crash
---

## Never Use && with Potentially Falsy Values

`value`가 빈 문자열이나 `0`이 될 수 있을 때는 `{value && <Component />}`를
절대 사용하지 않는다. 이 값들은 falsy지만 JSX가 렌더할 수 있다 — React Native가
`<Text>` 바깥에서 텍스트로 렌더하려다 production에서 hard crash를 일으킨다.

**Incorrect (crashes if count is 0 or name is ""):**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  return (
    <View>
      {name && <Text>{name}</Text>}
      {count && <Text>{count} items</Text>}
    </View>
  )
}
// If name="" or count=0, renders the falsy value → crash
```

**Correct (ternary with null):**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  return (
    <View>
      {name ? <Text>{name}</Text> : null}
      {count ? <Text>{count} items</Text> : null}
    </View>
  )
}
```

**Correct (explicit boolean coercion):**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  return (
    <View>
      {!!name && <Text>{name}</Text>}
      {!!count && <Text>{count} items</Text>}
    </View>
  )
}
```

**Best (early return):**

```tsx
function Profile({ name, count }: { name: string; count: number }) {
  if (!name) return null

  return (
    <View>
      <Text>{name}</Text>
      {count > 0 ? <Text>{count} items</Text> : null}
    </View>
  )
}
```

early return이 가장 명확하다. 인라인 조건을 쓰는 경우라면 ternary나 명시적인
boolean 검사를 선호한다.

**Lint rule:**
[eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react/blob/master/docs/rules/jsx-no-leaked-render.md)의
`react/jsx-no-leaked-render`를 활성화하면 자동으로 잡아낸다.
