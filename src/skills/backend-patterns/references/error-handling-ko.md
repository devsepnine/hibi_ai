# Error Handling

## Typed `ApiError`

```typescript
class ApiError extends Error {
  constructor(
    public statusCode: number,
    public message: string,
    public isOperational = true,
  ) {
    super(message)
    Object.setPrototypeOf(this, ApiError.prototype)
  }
}
```

`isOperational = true`는 예상된 에러(validation, not-found)를 표시한다. `false`는 버그를 의미한다 — 큰 소리로 로깅하고 알림을 보내며, 절대 삼키지 않는다.

## Centralized Handler

```typescript
export function errorHandler(error: unknown, req: Request): Response {
  if (error instanceof ApiError) {
    return NextResponse.json(
      { success: false, error: error.message },
      { status: error.statusCode },
    )
  }
  if (error instanceof z.ZodError) {
    return NextResponse.json(
      { success: false, error: 'Validation failed', details: error.errors },
      { status: 400 },
    )
  }
  console.error('Unexpected error:', error)
  return NextResponse.json(
    { success: false, error: 'Internal server error' },
    { status: 500 },
  )
}

// Usage in route
export async function GET(request: Request) {
  try {
    const data = await fetchData()
    return NextResponse.json({ success: true, data })
  } catch (error) {
    return errorHandler(error, request)
  }
}
```

## Retry with Exponential Backoff

**idempotent**한 작업(GET, PUT, DELETE)만 재시도한다. idempotency key 없이 POST를 재시도하지 않는다.

```typescript
async function fetchWithRetry<T>(fn: () => Promise<T>, maxRetries = 3): Promise<T> {
  let lastError: Error
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error as Error
      if (i < maxRetries - 1) {
        const delay = Math.pow(2, i) * 1000 // 1s, 2s, 4s
        await new Promise(r => setTimeout(r, delay))
      }
    }
  }
  throw lastError!
}
```

## Anti-Patterns

- `try { ... } catch (e) {}` — 조용한 삼키기. 항상 로깅하거나 다시 throw한다.
- `null`을 "에러"의 의미로 반환 — 호출자가 "not found"와 "DB down"을 구분할 수 없다.
- 문자열을 throw (`throw 'oops'`) — 스택 트레이스가 무용지물이 된다. 항상 `throw new Error(...)` 또는 서브클래스를 사용한다.
- `Error`를 catch하고 컨텍스트 없이 다시 throw — `new ApiError(500, 'Failed in X', { cause: e })`로 감싼다.
