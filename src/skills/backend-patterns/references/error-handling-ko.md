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

호출자에게 공개할 문구를 직접 작성한 4xx 오류에만 `expose`를 설정한다. 원래 예외는 서버에서 원인을 추적할 수 있도록 `cause`에 보관한다.

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

호출을 반복해도 안전하고 오류가 일시적일 때만 재시도한다. `isTransient`는 해당 서비스가 다시 시도할 수 있다고 문서화한 오류만 허용한다. `maxRetries`는 최초 실행 이후의 추가 시도 횟수다. 예시는 Node의 타이머를 사용하며 대기 시간은 최대 30초다.

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

- `try { ... } catch (e) {}`: 조용한 삼키기. 항상 로깅하거나 다시 throw한다.
- `null`을 "에러"의 의미로 반환: 호출자가 "not found"와 "DB down"을 구분할 수 없다.
- `throw 'oops'` 같은 문자열 throw: 스택 트레이스가 무용지물이 된다. 항상 `throw new Error(...)` 또는 서브클래스를 사용한다.
- 내부 오류에 맥락을 추가하려면 `new ApiError(500, 'Failed in X', false, { cause: e })`로 원인을 보존한다.
