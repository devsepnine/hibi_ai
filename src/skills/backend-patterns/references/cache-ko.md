# Caching Strategies

## Cache-Aside (default)

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

Redis 레이어로 어떤 repository든 데코레이트한다 — 캐시 관심사를 비즈니스 로직 밖에 둔다.

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
| Hot read, rare write (config, prices) | 30s–5m | 큰 QPS 이득을 위해 작은 stale을 허용 |
| User profile / session | 5–15m | 프로필 수정 시 invalidate |
| Aggregations (counts, leaderboards) | 1–10m | 게으르게 재계산 |
| Immutable artifacts (signed URLs) | until expiry | 리소스가 살아있는 동안만큼 캐시 |

## Invalidation Rules

- `update` / `delete` 시에는 항상 invalidate한다 — 정합성을 TTL에만 의존하지 않는다.
- flush 패턴보다 키 기반 invalidation (`del market:${id}`)을 선호한다.
- 리스트 캐시 (`markets:active`)의 경우, write 시 version key를 bump하여 오래된 키가 만료되도록 한다.
- 사용자별 데이터를 공유 키로 캐시하지 않는다 — 항상 `user:${id}:`로 네임스페이스를 구분한다.
