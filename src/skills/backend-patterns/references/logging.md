# Logging & Monitoring

## Structured Logger

```typescript
interface LogContext {
  userId?: string
  requestId?: string
  method?: string
  path?: string
  [key: string]: unknown
}

class Logger {
  log(level: 'info' | 'warn' | 'error', message: string, context?: LogContext) {
    const entry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      ...context,
    }
    console.log(JSON.stringify(entry))
  }
  info(msg: string, ctx?: LogContext) { this.log('info', msg, ctx) }
  warn(msg: string, ctx?: LogContext) { this.log('warn', msg, ctx) }
  error(msg: string, err: Error, ctx?: LogContext) {
    this.log('error', msg, { ...ctx, error: err.message, stack: err.stack })
  }
}

export const logger = new Logger()
```

## Request-Scoped Logging

Generate a `requestId` at the route boundary and pass it through every log line for that request — this is what makes a structured-log search actually useful.

```typescript
export async function GET(request: Request) {
  const requestId = crypto.randomUUID()
  logger.info('Fetching markets', { requestId, method: 'GET', path: '/api/markets' })
  try {
    const markets = await fetchMarkets()
    return NextResponse.json({ success: true, data: markets })
  } catch (error) {
    logger.error('Failed to fetch markets', error as Error, { requestId })
    return NextResponse.json({ error: 'Internal error' }, { status: 500 })
  }
}
```

For multi-service traces, propagate the id via header (`x-request-id`) and adopt it if present, else generate.

## What NOT to Log

- Secrets: API keys, JWTs, refresh tokens, DB passwords
- PII: full email (mask to `o***@example.com`), SSN, card numbers, address
- Request bodies that may carry the above — log only the field names / shape, not values
- Stack traces from user input (e.g. SQL fragments echoed back)

## Levels

| Level | Use for |
|---|---|
| `info` | Normal flow milestones (request received, job done) |
| `warn` | Recoverable anomalies (cache miss spike, retry succeeded after N attempts) |
| `error` | Failed request, unexpected exception, alert-worthy |

Avoid `debug` in production; gate behind env var if needed. Don't add `console.log` — it bypasses the structured pipeline.

## Shipping

- Stdout JSON → log collector (Datadog, Loki, CloudWatch, Logflare)
- Don't write to files from the app — let the platform handle rotation
- For Next.js on Vercel: `console.log(JSON.stringify(...))` is captured automatically
