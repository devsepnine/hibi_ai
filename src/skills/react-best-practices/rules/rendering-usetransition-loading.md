---
title: Track Search Transitions and Response Order
impact: LOW
impactDescription: reduces re-renders and improves code clarity
tags: rendering, transitions, useTransition, loading, state
---

## Track search transitions and response order

In React 19, `isPending` tracks pending transitions. Update the input immediately and only publish results or errors from the latest search.

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

The request number determines which response to apply. The surrounding error boundary handles current failures. Connect the API’s cancellation signal if earlier network work should also stop. [React useTransition](https://react.dev/reference/react/useTransition)
