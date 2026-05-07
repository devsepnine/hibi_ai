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

`isOperational = true` marks expected errors (validation, not-found). `false` means a bug — log loudly, alert, do not swallow.

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

Only retry **idempotent** operations (GET, PUT, DELETE). Never retry POST without an idempotency key.

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

- `try { ... } catch (e) {}` — silent swallow. Always log or rethrow.
- Returning `null` to mean "error" — caller can't distinguish "not found" from "DB down".
- Throwing strings (`throw 'oops'`) — stack trace is useless. Always `throw new Error(...)` or a subclass.
- Catching `Error` and re-throwing without context — wrap with `new ApiError(500, 'Failed in X', { cause: e })`.
