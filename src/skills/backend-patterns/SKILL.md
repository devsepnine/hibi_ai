---
name: backend-patterns
description: "Backend and API patterns for Node, Express, and Next.js routes: REST design, DB optimization, server-side structure. Use when writing server code or designing APIs. 백엔드 패턴, 서버 아키텍처, API 설계, DB 최적화."
---

# Backend Development Patterns

Scalable server-side patterns for Node.js / Express / Next.js API routes.

## When to Use

| Trigger | Section |
|---|---|
| Designing REST endpoints, resource layout | API Design |
| Repository, service, middleware structure | Layering |
| Slow queries, N+1, transactions | Database |
| Adding cache layer, such as Redis or in-memory | Caching → `references/cache.md` |
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

Rules: resource-based URLs with no verbs, plural nouns, `?key=value` for filter/sort/page, standard status codes 200/201/400/401/403/404/409/429/500.

### Response envelope

Preserve the existing wire contract for clients. For a new API that distinguishes success and failure in the body, the following union is one option.

```typescript
type ApiResponse<T> =
  | { success: true; data: T; meta?: { total: number; page: number; limit: number } }
  | { success: false; error: string; details?: unknown }
```

## Layering

Check the project architecture for the existing route, domain, and data access boundaries. For a new route, keep parsing and I/O at the boundary so calculations can be tested independently. Use the layers below when the service and repository have distinct responsibilities. [Layering examples](references/repo.md)

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

## Database

### Optimization Checklist
- Select only needed columns; never `SELECT *` on hot paths
- Add indexes for filter/sort/join columns
- Paginate everything that can grow unbounded
- Avoid N+1: batch-fetch related data, build a Map for `O(1)` lookup
- Use DB-side transactions, meaning RPC or stored proc, for multi-write atomicity

### N+1 Fix Pattern

```typescript
// Bad: 1 + N queries
for (const m of markets) m.creator = await getUser(m.creator_id)

// Good: 2 queries
const creators = await getUsers(markets.map(m => m.creator_id))
const usersById = new Map(creators.map(c => [c.id, c]))
const withCreators = markets.map(m => ({ ...m, creator: usersById.get(m.creator_id) }))
```

### Transactions
Wrap multi-table writes in a DB function such as Supabase RPC, Postgres `plpgsql`, or Prisma `$transaction`. Rollback on any failure. See `references/repo.md` for a Supabase RPC example.

## Caching

Read from a database that reflects committed writes when the caller needs current data. Use a shared cache for callers that accept stale reads. An older read finishing after invalidation can repopulate the old value for another TTL.

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

Full caching repository wrapper, TTL guidelines, invalidation strategies → `references/cache.md`.

## Error Handling

Use a typed `ApiError` and a central route error handler. Retry transient failures when repeating the operation is safe, and propagate cancellation promptly.

```typescript
// Both exports come from the error handling module shown in the reference.
import { ApiError, errorHandler } from './error-handling'

// at route boundary
try { return NextResponse.json({ success: true, data: await fetchData() }) }
catch (e) { return errorHandler(e, request) }
```

Centralized handler covering Zod errors and unknown errors, plus `fetchWithRetry` with 1s/2s/4s backoff → `references/error-handling.md`.

## Security

### Auth: JWT
```typescript
export async function requireAuth(request: Request) {
  const token = request.headers.get('authorization')?.replace('Bearer ', '')
  if (!token) throw new ApiError(401, 'Missing token', true)
  return jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload
}
```

### RBAC
Map roles → permissions, gate handlers via `requirePermission('delete')`. One source of truth for the role table.

### Rate Limiting
- Per identifier, meaning user id or IP, sliding window
- Reject with 429 + `Retry-After` when exceeded
- Use Redis for multi-instance deployments; in-memory only for single-process dev

On rejection, the limiter returns the wait until the next permitted request as a whole number of seconds.

```typescript
const decision = await limiter.checkLimit(ip, 100, 60_000)
if (!decision.allowed) {
  return NextResponse.json(
    { success: false, error: 'Rate limited' },
    { status: 429, headers: { 'Retry-After': String(decision.retryAfterSeconds) } },
  )
}
```

## Async Work

Check the response time budget and the impact of losing a job. When work must outlive the request, choose a queue that meets the project’s durability and retry requirements.

```typescript
await indexQueue.add({ marketId })
return NextResponse.json({ success: true, data: { queued: true } })
```

## Logging

Use the project’s structured logging format and request correlation ID. Keep secrets, tokens, and sensitive personal data out of logs.

```typescript
logger.info('Fetching markets', { requestId, method: 'GET', path: '/api/markets' })
logger.error('Failed to fetch markets', { requestId })
```

Full `Logger` class, request-id middleware, log-shipping notes → `references/logging.md`.

## References

- Node.js docs: https://nodejs.org/docs/latest/api/
- Express guide: https://expressjs.com/en/guide/routing.html
- Next.js Route Handlers: https://nextjs.org/docs/app/building-your-application/routing/route-handlers

Pick the smallest pattern that fits. Avoid premature middleware / queue / cache layers, and add them when a real bottleneck appears.
