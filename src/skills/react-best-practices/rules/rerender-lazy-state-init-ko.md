---
title: Use Lazy State Initialization
impact: MEDIUM
impactDescription: wasted computation on every render
tags: react, hooks, useState, performance, initialization
---

## Lazy State Initialization을 사용한다

비싼 초기값에는 함수를 `useState`에 전달한다. 함수 형태가 아니면 값이 단 한 번만 사용됨에도 매 렌더마다 initializer가 실행된다.

**잘못된 예 (매 렌더마다 실행):**

```tsx
function FilteredList({ items }: { items: Item[] }) {
  // buildSearchIndex() runs on EVERY render, even after initialization
  const [searchIndex, setSearchIndex] = useState(buildSearchIndex(items))
  const [query, setQuery] = useState('')
  
  // When query changes, buildSearchIndex runs again unnecessarily
  return <SearchResults index={searchIndex} query={query} />
}

function UserProfile() {
  // JSON.parse runs on every render
  const [settings, setSettings] = useState(
    JSON.parse(localStorage.getItem('settings') || '{}')
  )
  
  return <SettingsForm settings={settings} onChange={setSettings} />
}
```

**올바른 예 (한 번만 실행):**

```tsx
function FilteredList({ items }: { items: Item[] }) {
  // buildSearchIndex() runs ONLY on initial render
  const [searchIndex, setSearchIndex] = useState(() => buildSearchIndex(items))
  const [query, setQuery] = useState('')
  
  return <SearchResults index={searchIndex} query={query} />
}

function UserProfile() {
  // JSON.parse runs only on initial render
  const [settings, setSettings] = useState(() => {
    const stored = localStorage.getItem('settings')
    return stored ? JSON.parse(stored) : {}
  })
  
  return <SettingsForm settings={settings} onChange={setSettings} />
}
```

localStorage/sessionStorage에서 초기값 계산, 자료구조 빌드(인덱스, 맵), DOM 읽기, 무거운 변환 시에는 lazy 초기화를 사용한다.

단순 primitive (`useState(0)`), 직접 참조 (`useState(props.value)`), 가벼운 리터럴 (`useState({})`)에는 함수 형태가 불필요하다.
