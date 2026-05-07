---
name: backend-patterns
description: Backend architecture patterns, API design, database optimization, and server-side best practices for Node.js, Express, and Next.js API routes. Use when writing server-side code, designing REST APIs, 백엔드 패턴, 서버 아키텍처, API 설계, 데이터베이스 최적화.
keywords: [backend, api, server, 백엔드, 서버, rest, node, nextjs]
---

# 백엔드 개발 패턴

Node.js / Express / Next.js API 라우트를 위한 확장 가능한 서버 사이드 패턴.

## 사용 시점

| 트리거 | 섹션 |
|---|---|
| REST 엔드포인트 설계, 리소스 레이아웃 | API Design |
| Repository, service, middleware 구조 | Layering |
| 느린 쿼리, N+1, 트랜잭션 | Database |
| 캐시 레이어 추가 (Redis, in-memory) | Caching → `references/cache.md` |
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

규칙: 리소스 기반 URL (동사 금지), 복수 명사, 필터/정렬/페이지에 `?key=value` 사용, 표준 상태 코드 (200/201/400/401/403/404/409/429/500) 사용.

### 응답 봉투 (Response Envelope)

```typescript
type ApiResponse<T> = {
  success: boolean
  data?: T
  error?: string
  meta?: { total: number; page: number; limit: number }
}
```

## 레이어링

3계층 분리: **Route → Service → Repository**. 라우트는 파싱/검증, 서비스는 비즈니스 로직, 리포지토리는 데이터 접근을 담당한다. 전체 Repository + Service + Middleware 코드 샘플은 `references/repo.md` 참조.

```typescript
// repo: data access only
interface MarketRepository {
  findAll(filters?: MarketFilters): Promise<Market[]>
  findById(id: string): Promise<Market | null>
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
- 필요한 컬럼만 SELECT (핫 패스에서 절대 `SELECT *` 금지)
- 필터/정렬/조인 컬럼에 인덱스 추가
- 무한정 커질 수 있는 모든 것은 페이지네이션
- N+1 회피: 관련 데이터를 일괄 조회하고 O(1) 룩업을 위해 Map 구성
- 다중 쓰기 원자성을 위해 DB 측 트랜잭션 (RPC / stored proc) 사용

### N+1 수정 패턴

```typescript
// Bad: 1 + N queries
for (const m of markets) m.creator = await getUser(m.creator_id)

// Good: 2 queries
const creators = await getUsers(markets.map(m => m.creator_id))
const map = new Map(creators.map(c => [c.id, c]))
markets.forEach(m => { m.creator = map.get(m.creator_id) })
```

### 트랜잭션
다중 테이블 쓰기는 DB 함수 (Supabase RPC / Postgres `plpgsql` / Prisma `$transaction`) 로 감싼다. 어떤 실패에도 롤백한다. Supabase RPC 예제는 `references/repo.md` 참조.

## 캐싱

Cache-aside가 기본 패턴: 캐시 읽기 → 미스 → DB → TTL과 함께 캐시 백필 → 반환. 쓰기 시 무효화한다.

```typescript
async function getMarket(id: string) {
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

타입화된 `ApiError` + 라우트당 단일 `errorHandler`를 사용한다. 절대 에러를 조용히 삼키지 말 것. 멱등성 호출은 지수 백오프로 재시도한다.

```typescript
class ApiError extends Error {
  constructor(public statusCode: number, message: string) { super(message) }
}

// at route boundary
try { return NextResponse.json({ success: true, data: await fetchData() }) }
catch (e) { return errorHandler(e, request) }
```

중앙 핸들러 (Zod 에러, unknown 에러), `fetchWithRetry` (1s/2s/4s 백오프) → `references/error-handling.md`.

## 보안

### 인증 (JWT)
```typescript
export async function requireAuth(request: Request) {
  const token = request.headers.get('authorization')?.replace('Bearer ', '')
  if (!token) throw new ApiError(401, 'Missing token')
  return jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload
}
```

### RBAC
역할 → 권한 매핑, `requirePermission('delete')` 로 핸들러 게이팅. 역할 테이블은 단일 진실 공급원으로 둔다.

### Rate Limiting
- 식별자별 (user id / IP), 슬라이딩 윈도우
- 초과 시 429 + `Retry-After` 로 거절
- 다중 인스턴스 배포에는 Redis 사용; 단일 프로세스 dev에서만 in-memory 허용

```typescript
const allowed = await limiter.checkLimit(ip, 100, 60_000) // 100 req/min
if (!allowed) return NextResponse.json({ error: 'Rate limited' }, { status: 429 })
```

## 비동기 작업

긴 작업 (>200ms) 은 큐로 위임하고 즉시 응답한다. 단일 프로세스 큐는 dev / 저용량에는 OK; 프로덕션은 Redis 기반 (BullMQ) 또는 외부 (SQS, Cloud Tasks) 가 필요하다.

```typescript
await indexQueue.add({ marketId })
return NextResponse.json({ success: true, message: 'Job queued' })
```

## 로깅

항상 JSON 구조화, 추적을 위해 항상 `requestId` (요청당 UUID) 포함. 로그 레벨: `info` / `warn` / `error`. 시크릿, 토큰, 또는 전체 PII를 로깅하지 말 것.

```typescript
logger.info('Fetching markets', { requestId, method, path })
logger.error('Failed', err, { requestId })
```

전체 `Logger` 클래스, request-id 미들웨어, 로그 전송 노트는 `references/logging.md` 참조.

## 참고 자료

- Node.js docs: https://nodejs.org/docs/latest/api/
- Express guide: https://expressjs.com/en/guide/routing.html
- Next.js Route Handlers: https://nextjs.org/docs/app/building-your-application/routing/route-handlers

요구사항에 맞는 가장 작은 패턴을 선택한다. 미들웨어 / 큐 / 캐시 레이어를 성급하게 추가하지 말 것 — 실제 병목이 나타나면 추가한다.
