# Caching Strategies

## Cache-Aside, the default

Read path: cache → on miss, DB → backfill → return. Write path: DB → invalidate cache.

```typescript
async function getMarketWithCache(id: string): Promise<Market> {
  const cacheKey = `market:${id}`
  const cached = await redis.get(cacheKey)
  if (cached) return JSON.parse(cached)
  const market = await db.markets.findUnique({ where: { id } })
  if (!market) throw new Error('Market not found')
  await redis.setex(cacheKey, 300, JSON.stringify(market))
  return market
}
```

## Caching Repository Wrapper

Decorate any repository with a Redis layer, which keeps the cache concern out of business logic.

```typescript
class CachedMarketRepository implements MarketRepository {
  constructor(private base: MarketRepository, private redis: RedisClient) {}

  async findById(id: string): Promise<Market | null> {
    const cached = await this.redis.get(`market:${id}`)
    if (cached) return JSON.parse(cached)
    const market = await this.base.findById(id)
    if (market) await this.redis.setex(`market:${id}`, 300, JSON.stringify(market))
    return market
  }

  async invalidateCache(id: string): Promise<void> {
    await this.redis.del(`market:${id}`)
  }
}
```

## TTL Guidelines

| Data shape | TTL | Notes |
|---|---|---|
| Hot read, rare write, such as config and prices | 30s to 5m | Tolerate small staleness for big QPS win |
| User profile / session | 5 to 15m | Invalidate on profile edit |
| Aggregations such as counts and leaderboards | 1 to 10m | Recompute lazily |
| Immutable artifacts such as signed URLs | until expiry | Cache for as long as the resource lives |

## Invalidation Rules

- Always invalidate on `update` / `delete`, and don't rely on TTL alone for correctness.
- Prefer key-based invalidation, such as `del market:${id}`, over flush patterns.
- For list caches such as `markets:active`, bump a version key on any write so old keys age out.
- Never cache user-specific data under a shared key; always namespace by `user:${id}:`.
