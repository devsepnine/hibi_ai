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

라우트 경계에서 `requestId`를 생성하고 해당 요청에 대한 모든 로그 라인에 통과시킨다 — 이것이 structured-log 검색을 실제로 유용하게 만든다.

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

다중 서비스 trace의 경우, 헤더(`x-request-id`)를 통해 id를 전파하고 있으면 채택, 없으면 생성한다.

## What NOT to Log

- Secrets: API key, JWT, refresh token, DB password
- PII: 전체 이메일 (`o***@example.com`으로 마스킹), 주민번호, 카드 번호, 주소
- 위 항목을 담을 수 있는 request body — 값이 아닌 필드 이름/shape만 로깅한다
- 사용자 입력에서 온 스택 트레이스 (예: 다시 echo된 SQL 단편)

## Levels

| Level | Use for |
|---|---|
| `info` | 정상 흐름 마일스톤 (요청 수신, 작업 완료) |
| `warn` | 복구 가능한 이상 (캐시 미스 급증, N번 재시도 후 성공) |
| `error` | 실패한 요청, 예상치 못한 예외, 알림 가치 있음 |

프로덕션에서 `debug`를 피한다; 필요하면 환경 변수 뒤로 게이팅한다. `console.log`를 추가하지 않는다 — structured 파이프라인을 우회한다.

## Shipping

- Stdout JSON → log collector (Datadog, Loki, CloudWatch, Logflare)
- 앱에서 파일에 쓰지 않는다 — 플랫폼이 rotation을 처리하도록 한다
- Vercel의 Next.js의 경우: `console.log(JSON.stringify(...))`가 자동으로 캡처된다
