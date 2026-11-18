---
name: backend-patterns
description: "Backend and API patterns for Node, Express, and Next.js routes: REST design, DB optimization, server-side structure. Use when writing server code or designing APIs. 백엔드 패턴, 서버 아키텍처, API 설계, DB 최적화."
---

# 백엔드 개발 패턴

Node.js / Express / Next.js API 라우트를 위한 확장 가능한 서버 사이드 패턴.

## 사용 시점

| 트리거 | 섹션 |
|---|---|
| REST 엔드포인트 설계, 리소스 레이아웃 | API Design |
| Repository, service, middleware 구조 | Layering |
| 느린 쿼리, N+1, 트랜잭션 | Database |
| 캐시 레이어 추가, 예를 들어 Redis나 in-memory | Caching → `references/cache.md` |
| 중앙 집중식 에러, 재시도 | Errors → `references/error-handling.md` |
| JWT 인증, RBAC, rate limit | Security |
| 백그라운드 작업, 큐 | Async |
| 구조화 로그, request ID | Logging → `references/logging.md` |

## API 설계

### REST 컨벤션

```
GET    /api/markets           # list (filters via ?status=&sort=&limit=&offset=)
GET    /api/markets/:id       # read
POST   /api/markets           # create
PUT    /api/markets/:id       # replace
PATCH  /api/markets/:id       # partial update
DELETE /api/markets/:id       # delete
```

규칙: 리소스 기반 URL로 동사 금지, 복수 명사, 필터/정렬/페이지에 `?key=value` 사용, 표준 상태 코드 200/201/400/401/403/404/409/429/500 사용.

### 응답 형식

기존 엔드포인트는 클라이언트와 합의한 응답 형식을 유지한다. 새 API에서 성공과 실패를 응답 본문으로 구분할 때는 아래처럼 표현할 수 있다.

```typescript
type ApiResponse<T> =
  | { success: true; data: T; meta?: { total: number; page: number; limit: number } }
  | { success: false; error: string; details?: unknown }
```

## 레이어링

프로젝트의 아키텍처 문서에서 라우트, 도메인 로직, 데이터 접근의 경계를 먼저 확인한다. 새 경로에서는 입력 파싱과 I/O를 경계에 두고 계산 로직을 독립적으로 테스트한다. 서비스와 리포지토리를 분리할 책임이 있을 때 아래 구조를 사용한다. [계층별 예시](references/repo-ko.md)

```typescript
// repo: data access only
interface MarketRepository {
  findAll(filters?: MarketFilters): Promise<Market[]>
  findById(id: string): Promise<Market | null>
  findByIds(ids: readonly string[]): Promise<readonly Market[]>
  create(data: CreateMarketDto): Promise<Market>
  update(id: string, data: UpdateMarketDto): Promise<Market>
  delete(id: string): Promise<void>
}

// service: orchestration + business rules
class MarketService {
  constructor(private repo: MarketRepository) {}
  // ... no SQL, no HTTP here
}

// middleware: cross-cutting (auth, logging, rate-limit)
export const withAuth = (handler) => async (req, res) => {
  const user = await verifyToken(req.headers.authorization)
  req.user = user
  return handler(req, res)
}
```

## 데이터베이스

### 최적화 체크리스트
- 필요한 컬럼만 SELECT; 핫 패스에서 절대 `SELECT *` 금지
- 필터/정렬/조인 컬럼에 인덱스 추가
- 무한정 커질 수 있는 모든 것은 페이지네이션
- N+1 회피: 관련 데이터를 일괄 조회하고 `O(1)` 룩업을 위해 Map 구성
- 다중 쓰기 원자성을 위해 DB 측 트랜잭션, 곧 RPC나 stored proc 사용

### N+1 수정 패턴

```typescript
// Bad: 1 + N queries
for (const m of markets) m.creator = await getUser(m.creator_id)

// Good: 2 queries
const creators = await getUsers(markets.map(m => m.creator_id))
const usersById = new Map(creators.map(c => [c.id, c]))
const withCreators = markets.map(m => ({ ...m, creator: usersById.get(m.creator_id) }))
```

### 트랜잭션
다중 테이블 쓰기는 Supabase RPC, Postgres `plpgsql`, Prisma `$transaction` 같은 DB 함수로 감싼다. 어떤 실패에도 롤백한다. Supabase RPC 예제는 `references/repo.md` 참조.

## 캐싱

쓰기 완료 후 최신 값이 필요한 조회는 쓰기를 반영한 DB에서 읽는다. 지연을 허용하는 조회만 공유 캐시를 사용한다. 쓰기 후 키를 삭제해도 이전 조회가 마지막에 끝나면 한 TTL 동안 구버전을 다시 저장할 수 있다.

```typescript
async function getMarket(id: string, freshness: 'current' | 'allow-stale' = 'current'): Promise<Market | null> {
  if (freshness === 'current') return db.markets.findUnique({ where: { id } })
  const key = `market:${id}`
  const hit = await redis.get(key)
  if (hit) return JSON.parse(hit)
  const m = await db.markets.findUnique({ where: { id } })
  if (m) await redis.setex(key, 300, JSON.stringify(m))
  return m
}
```

전체 캐싱 리포지토리 래퍼, TTL 가이드라인, 무효화 전략은 `references/cache.md` 참조.

## 에러 처리

타입화된 `ApiError`와 중앙 오류 처리기를 사용한다. 반복해도 안전한 작업의 일시적인 오류만 재시도하고 취소는 즉시 전달한다.

```typescript
// ApiError와 errorHandler는 아래 참고 문서의 같은 모듈에서 가져온다.
import { ApiError, errorHandler } from './error-handling'

// at route boundary
try { return NextResponse.json({ success: true, data: await fetchData() }) }
catch (e) { return errorHandler(e, request) }
```

Zod 에러와 unknown 에러를 다루는 중앙 핸들러, 그리고 1s/2s/4s 백오프의 `fetchWithRetry` → `references/error-handling.md`.

## 보안

### 인증: JWT
```typescript
export async function requireAuth(request: Request) {
  const token = request.headers.get('authorization')?.replace('Bearer ', '')
  if (!token) throw new ApiError(401, 'Missing token', true)
  return jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload
}
```

### RBAC
역할 → 권한 매핑, `requirePermission('delete')` 로 핸들러 게이팅. 역할 테이블은 단일 진실 공급원으로 둔다.

### Rate Limiting
- 식별자별, 곧 user id나 IP, 슬라이딩 윈도우
- 초과 시 429 + `Retry-After` 로 거절
- 다중 인스턴스 배포에는 Redis 사용; 단일 프로세스 dev에서만 in-memory 허용

차단 시 limiter가 다음 요청까지 기다릴 시간을 초 단위 정수로 반환하도록 한다.

```typescript
const decision = await limiter.checkLimit(ip, 100, 60_000)
if (!decision.allowed) {
  return NextResponse.json(
    { success: false, error: 'Rate limited' },
    { status: 429, headers: { 'Retry-After': String(decision.retryAfterSeconds) } },
  )
}
```

## 비동기 작업

응답 시간 한도와 작업 유실 시의 영향을 확인한다. 요청 이후에도 실행해야 하는 작업은 프로젝트의 지속성·재시도 요건을 충족하는 큐에 맡긴다.

```typescript
await indexQueue.add({ marketId })
return NextResponse.json({ success: true, data: { queued: true } })
```

## 로깅

프로젝트의 구조화 로그 형식과 요청 추적 ID를 사용한다. 시크릿, 토큰, 민감한 개인정보는 로그에서 제외한다.

```typescript
logger.info('Fetching markets', { requestId, method: 'GET', path: '/api/markets' })
logger.error('Failed to fetch markets', { requestId })
```

전체 `Logger` 클래스, request-id 미들웨어, 로그 전송 노트는 `references/logging.md` 참조.

## 참고 자료

- Node.js docs: https://nodejs.org/docs/latest/api/
- Express guide: https://expressjs.com/en/guide/routing.html
- Next.js Route Handlers: https://nextjs.org/docs/app/building-your-application/routing/route-handlers

요구사항에 맞는 가장 작은 패턴을 선택한다. 미들웨어 / 큐 / 캐시 레이어를 성급하게 추가하지 말고, 실제 병목이 나타나면 추가한다.
