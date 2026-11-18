# Error Handling

## Typed `ApiError`

```typescript
export class ApiError extends Error {
  constructor(
    readonly statusCode: number,
    message: string,
    readonly expose = false,
    options?: ErrorOptions,
  ) {
    super(message, options)
    Object.setPrototypeOf(this, ApiError.prototype)
  }
}
```

Set `expose` only for a 4xx error with a message written for the client. Keep the original exception in `cause` for server diagnostics.

## Centralized Handler

```typescript
export function errorHandler(error: unknown, req: Request): Response {
  if (error instanceof ApiError && error.statusCode >= 400 && error.statusCode < 500) {
    return NextResponse.json(
      { success: false, error: error.expose ? error.message : 'Request failed' },
      { status: error.statusCode },
    )
  }
  if (error instanceof z.ZodError) {
    return NextResponse.json(
      { success: false, error: 'Validation failed' },
      { status: 400 },
    )
  }
  const status = error instanceof ApiError && error.statusCode >= 500 && error.statusCode < 600
    ? error.statusCode
    : 500
  console.error('Unexpected route error', { method: req.method, status })
  return NextResponse.json(
    { success: false, error: 'Internal server error' },
    { status },
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

Retry only when repeating the operation is safe and the failure is transient. Have `isTransient` recognize the retryable failures documented by the service. `maxRetries` counts additional attempts after the first. This Node example caps each backoff at 30 seconds.

```typescript
import { setTimeout as delay } from 'node:timers/promises'

async function fetchWithRetry<T>(
  fn: (signal?: AbortSignal) => Promise<T>,
  options: { isTransient: (error: unknown) => boolean; maxRetries?: number; signal?: AbortSignal },
): Promise<T> {
  const { isTransient, maxRetries = 3, signal } = options
  if (!Number.isSafeInteger(maxRetries) || maxRetries < 0) {
    throw new RangeError('maxRetries must be a nonnegative safe integer')
  }
  for (let retries = 0; ; retries++) {
    signal?.throwIfAborted()
    try {
      return await fn(signal)
    } catch (error) {
      signal?.throwIfAborted()
      if (error instanceof Error && error.name === 'AbortError') throw error
      if (retries >= maxRetries || !isTransient(error)) throw error
      await delay(Math.min(1000 * 2 ** retries, 30_000), undefined, { signal })
    }
  }
}
```

## Anti-Patterns

- `try { ... } catch (e) {}`: silent swallow. Always log or rethrow.
- Returning `null` to mean "error": caller can't distinguish "not found" from "DB down".
- Throwing strings, like `throw 'oops'`: stack trace is useless. Always `throw new Error(...)` or a subclass.
- Preserve the cause when adding context to an internal failure with `new ApiError(500, 'Failed in X', false, { cause: e })`.
