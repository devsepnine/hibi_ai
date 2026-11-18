---
title: 검색 전환과 응답 순서 관리
impact: LOW
impactDescription: reduces re-renders and improves code clarity
tags: rendering, transitions, useTransition, loading, state
---

## 검색 전환과 응답 순서 관리

React 19에서 `isPending`은 진행 중인 전환을 나타낸다. 검색어를 즉시 갱신하고 최신 요청의 결과와 오류만 화면에 반영한다.

```tsx
import { useRef, useState, useTransition } from 'react'

function SearchResults() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<string[]>([])
  const [isPending, startTransition] = useTransition()
  const latestRequest = useRef(0)

  function handleSearch(value: string) {
    setQuery(value)
    const request = ++latestRequest.current
    startTransition(async () => {
      try {
        const data = await fetchResults(value)
        if (request === latestRequest.current) {
          startTransition(() => setResults(data))
        }
      } catch (error) {
        if (request === latestRequest.current) throw error
      }
    })
  }

  return (
    <>
      <input value={query} onChange={event => handleSearch(event.target.value)} />
      {isPending && <Spinner />}
      <ResultsList results={results} />
    </>
  )
}
```

요청 번호가 응답 순서를 판별한다. 최신 요청의 오류는 주변 Error Boundary가 처리한다. 이전 네트워크 작업까지 중단해야 하면 해당 API의 취소 신호도 연결한다. [React useTransition](https://react.dev/reference/react/useTransition)
