# Logging & Monitoring

## Structured Logger

```typescript
interface LogContext {
  userId?: string
  requestId?: string
  method?: string
  path?: string
}

class Logger {
  log(level: 'info' | 'warn' | 'error', message: string, context?: LogContext) {
    const entry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      requestId: context?.requestId,
      method: context?.method,
      path: context?.path,
      userId: context?.userId,
    }
    console.log(JSON.stringify(entry))
  }
  info(msg: string, ctx?: LogContext) { this.log('info', msg, ctx) }
  warn(msg: string, ctx?: LogContext) { this.log('warn', msg, ctx) }
  error(msg: string, ctx?: LogContext) {
    this.log('error', msg, ctx)
  }
}

export const logger = new Logger()
```

이 로거에는 코드에서 정한 이벤트 메시지와 기록이 허용된 식별자, HTTP 메서드, 라우트 템플릿을 전달한다. 원본 요청 URL이나 예외의 메시지와 스택에는 자격 증명이 들어 있을 수 있다.

## Request-Scoped Logging

라우트 경계에서 `requestId`를 생성하고 해당 요청에 대한 모든 로그 라인에 통과시킨다. 이것이 structured-log 검색을 실제로 유용하게 만든다.

```typescript
export async function GET(request: Request) {
  const requestId = crypto.randomUUID()
  logger.info('Fetching markets', { requestId, method: 'GET', path: '/api/markets' })
  try {
    const markets = await fetchMarkets()
    return NextResponse.json({ success: true, data: markets })
  } catch {
    logger.error('Failed to fetch markets', { requestId })
    return NextResponse.json({ error: 'Internal error' }, { status: 500 })
  }
}
```

하위 서비스에도 요청 ID를 전달한다. 형식을 검증하는 신뢰할 수 있는 게이트웨이의 ID를 사용하거나 요청 경계에서 새로 생성한다.

## What NOT to Log

- Secrets: API key, JWT, refresh token, DB password
- PII: 전체 이메일은 `o***@example.com`으로 마스킹, 주민번호, 카드 번호, 주소
- 위 항목을 담을 수 있는 request body: 값이 아닌 필드 이름/shape만 로깅한다
- 사용자 입력에서 온 스택 트레이스, 예를 들어 다시 echo된 SQL 단편

## Levels

| Level | Use for |
|---|---|
| `info` | 정상 흐름 마일스톤, 예를 들어 요청 수신이나 작업 완료 |
| `warn` | 복구 가능한 이상, 예를 들어 캐시 미스 급증이나 N번 재시도 후 성공 |
| `error` | 실패한 요청, 예상치 못한 예외, 알림 가치 있음 |

프로덕션에서 `debug`를 피한다; 필요하면 환경 변수 뒤로 게이팅한다. `console.log`를 추가하지 않는다. structured 파이프라인을 우회하기 때문이다.

## Shipping

- Stdout JSON → Datadog, Loki, CloudWatch, Logflare 같은 log collector
- 앱에서 파일에 쓰지 않는다. 플랫폼이 rotation을 처리하도록 한다
- Vercel의 Next.js의 경우: `console.log(JSON.stringify(...))`가 자동으로 캡처된다
