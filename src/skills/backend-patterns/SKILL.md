---
name: backend-patterns
description: Backend architecture patterns, API design, database optimization, and server-side best practices for Node.js, Express, and Next.js API routes. Use when writing server-side code, designing REST APIs, 백엔드 패턴, 서버 아키텍처, API 설계, 데이터베이스 최적화.
keywords: [backend, api, server, 백엔드, 서버, rest, node, nextjs]
---

# Backend Development Patterns

Scalable server-side patterns for Node.js / Express / Next.js API routes.

## When to Use

| Trigger | Section |
|---|---|
| Designing REST endpoints, resource layout | API Design |
| Repository, service, middleware structure | Layering |
| Slow queries, N+1, transactions | Database |
| Adding cache layer (Redis, in-memory) | Caching → `references/cache.md` |
| Centralized errors, retries | Errors → `references/error-handling.md` |
| JWT auth, RBAC, rate limit | Security |
| Background jobs, queues | Async |
| Structured logs, request IDs | Logging → `references/logging.md` |

## API Design

### REST Conventions

```
GET    /api/markets           # list (filters via ?status=&sort=&limit=&offset=)
GET    /api/markets/:id       # read
POST   /api/markets           # create
PUT    /api/markets/:id       # replace
PATCH  /api/markets/:id       # partial update
DELETE /api/markets/:id       # delete
```

Rules: resource-based URLs (no verbs), plural nouns, `?key=value` for filter/sort/page, standard status codes (200/201/400/401/403/404/409/429/500).

### Response Envelope

```typescript
type ApiResponse<T> = {
  success: boolean
  data?: T
  error?: string
  meta?: { total: number; page: number; limit: number }
}
```

## Layering

Three-tier separation: **Route → Service → Repository**. Routes parse/validate, services own business logic, repositories own data access. See `references/repo.md` for full Repository + Service + Middleware code samples.

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

## Database

### Optimization Checklist
- Select only needed columns (never `SELECT *` on hot paths)
- Add indexes for filter/sort/join columns
- Paginate everything that can grow unbounded
- Avoid N+1: batch-fetch related data, build a Map for O(1) lookup
- Use DB-side transactions (RPC / stored proc) for multi-write atomicity

### N+1 Fix Pattern

```typescript
// Bad: 1 + N queries
for (const m of markets) m.creator = await getUser(m.creator_id)

// Good: 2 queries
const creators = await getUsers(markets.map(m => m.creator_id))
const map = new Map(creators.map(c => [c.id, c]))
markets.forEach(m => { m.creator = map.get(m.creator_id) })
```

### Transactions
Wrap multi-table writes in a DB function (Supabase RPC / Postgres `plpgsql` / Prisma `$transaction`). Rollback on any failure. See `references/repo.md` for a Supabase RPC example.

## Caching

Cache-aside is the default pattern: read cache → miss → DB → backfill cache with TTL → return. Invalidate on write.

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

Full caching repository wrapper, TTL guidelines, invalidation strategies → `references/cache.md`.

## Error Handling

Use a typed `ApiError` + a single `errorHandler` per route. Never swallow errors silently. Retry idempotent calls with exponential backoff.

```typescript
class ApiError extends Error {
  constructor(public statusCode: number, message: string) { super(message) }
}

// at route boundary
try { return NextResponse.json({ success: true, data: await fetchData() }) }
catch (e) { return errorHandler(e, request) }
```

Centralized handler (Zod errors, unknown errors), `fetchWithRetry` (1s/2s/4s backoff) → `references/error-handling.md`.

## Security

### Auth (JWT)
```typescript
export async function requireAuth(request: Request) {
  const token = request.headers.get('authorization')?.replace('Bearer ', '')
  if (!token) throw new ApiError(401, 'Missing token')
  return jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload
}
```

### RBAC
Map roles → permissions, gate handlers via `requirePermission('delete')`. One source of truth for the role table.

### Rate Limiting
- Per identifier (user id / IP), sliding window
- Reject with 429 + `Retry-After` when exceeded
- Use Redis for multi-instance deployments; in-memory only for single-process dev

```typescript
const allowed = await limiter.checkLimit(ip, 100, 60_000) // 100 req/min
if (!allowed) return NextResponse.json({ error: 'Rate limited' }, { status: 429 })
```

## Async Work

Offload long tasks (>200ms) to a queue; respond immediately. Single-process queue is fine for dev / low volume; production needs Redis-backed (BullMQ) or external (SQS, Cloud Tasks).

```typescript
await indexQueue.add({ marketId })
return NextResponse.json({ success: true, message: 'Job queued' })
```

## Logging

Always JSON-structured, always include `requestId` (UUID per request) for tracing. Log levels: `info` / `warn` / `error`. Never log secrets, tokens, or full PII.

```typescript
logger.info('Fetching markets', { requestId, method, path })
logger.error('Failed', err, { requestId })
```

Full `Logger` class, request-id middleware, log-shipping notes → `references/logging.md`.

## References

- Node.js docs: https://nodejs.org/docs/latest/api/
- Express guide: https://expressjs.com/en/guide/routing.html
- Next.js Route Handlers: https://nextjs.org/docs/app/building-your-application/routing/route-handlers

Pick the smallest pattern that fits. Avoid premature middleware / queue / cache layers — add them when a real bottleneck appears.
