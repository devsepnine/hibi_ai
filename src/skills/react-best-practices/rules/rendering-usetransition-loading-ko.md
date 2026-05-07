---
title: Use useTransition Over Manual Loading States
impact: LOW
impactDescription: reduces re-renders and improves code clarity
tags: rendering, transitions, useTransition, loading, state
---

## 수동 loading state 대신 useTransition을 사용한다

loading 상태에 대해 수동 `useState` 대신 `useTransition`을 사용한다. 이는 빌트인 `isPending` 상태를 제공하고 transition을 자동으로 관리한다.

**잘못된 예 (수동 loading state):**

```tsx
function SearchResults() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState([])
  const [isLoading, setIsLoading] = useState(false)

  const handleSearch = async (value: string) => {
    setIsLoading(true)
    setQuery(value)
    const data = await fetchResults(value)
    setResults(data)
    setIsLoading(false)
  }

  return (
    <>
      <input onChange={(e) => handleSearch(e.target.value)} />
      {isLoading && <Spinner />}
      <ResultsList results={results} />
    </>
  )
}
```

**올바른 예 (빌트인 pending 상태가 있는 useTransition):**

```tsx
import { useTransition, useState } from 'react'

function SearchResults() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState([])
  const [isPending, startTransition] = useTransition()

  const handleSearch = (value: string) => {
    setQuery(value) // Update input immediately
    
    startTransition(async () => {
      // Fetch and update results
      const data = await fetchResults(value)
      setResults(data)
    })
  }

  return (
    <>
      <input onChange={(e) => handleSearch(e.target.value)} />
      {isPending && <Spinner />}
      <ResultsList results={results} />
    </>
  )
}
```

**이점:**

- **자동 pending 상태**: `setIsLoading(true/false)`를 수동으로 관리할 필요가 없다
- **에러 회복력**: transition이 throw하더라도 pending 상태가 정확히 리셋된다
- **더 나은 반응성**: 업데이트 도중에도 UI가 반응성을 유지한다
- **인터럽트 처리**: 새 transition이 시작되면 진행 중이던 transition을 자동으로 취소한다

참고: [useTransition](https://react.dev/reference/react/useTransition)
