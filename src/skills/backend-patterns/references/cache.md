# Caching Strategies

## Freshness required by the caller

For a current read, use a database that reflects committed writes. Cache only when the caller accepts stale data. Even after a writer clears the key, a read already in flight can refill it with an older value and a fresh TTL.

```typescript
async function getMarket(id: string, freshness: 'current' | 'allow-stale' = 'current'): Promise<Market | null> {
  const cacheKey = `market:${id}`
  if (freshness === 'allow-stale') {
    const cached = await redis.get(cacheKey)
    if (cached) return JSON.parse(cached)
  }
  const market = await db.markets.findUnique({ where: { id } })
  if (!market) return null
  if (freshness === 'allow-stale') await redis.setex(cacheKey, 300, JSON.stringify(market))
  return market
}
```

## Caching Repository Wrapper

Decorate any repository with a Redis layer, which keeps the cache concern out of business logic.

```typescript
class CachedMarketReader {
  constructor(private base: MarketRepository, private redis: RedisClient) {}

  async findById(id: string, freshness: 'current' | 'allow-stale' = 'current'): Promise<Market | null> {
    if (freshness === 'current') return this.base.findById(id)
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
| Profile display data | 5 to 15 minutes | Invalidate after edits and check access against current state |
| Aggregations such as counts and leaderboards | 1 to 10m | Recompute lazily |
| Immutable artifacts such as signed URLs | until expiry | Cache for as long as the resource lives |

## Invalidation Rules

- Always invalidate on `update` / `delete`, and don't rely on TTL alone for correctness.
- Prefer key-based invalidation, such as `del market:${id}`, over flush patterns.
- For list caches such as `markets:active`, bump a version key on any write so old keys age out.
- Never cache user-specific data under a shared key; always namespace by `user:${id}:`.
