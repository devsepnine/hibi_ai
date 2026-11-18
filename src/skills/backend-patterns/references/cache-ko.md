# Caching Strategies

## 호출자가 허용하는 지연

최신 값이 필요하면 쓰기를 반영한 DB에서 직접 읽는다. 이전 값을 허용하는 호출에서만 캐시를 사용한다. DB 쓰기 후 키를 지워도 먼저 시작한 조회가 늦게 끝나면 이전 값이 새 TTL로 캐싱된다.

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

Redis 레이어로 어떤 repository든 데코레이트한다. 캐시 관심사를 비즈니스 로직 밖에 둔다.

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
| Hot read, rare write, such as config and prices | 30s to 5m | 큰 QPS 이득을 위해 작은 stale을 허용 |
| 프로필 표시 데이터 | 5분에서 15분 | 프로필 수정 후 무효화하며 접근 권한 판정은 최신 상태에서 수행 |
| Aggregations such as counts and leaderboards | 1 to 10m | 게으르게 재계산 |
| Immutable artifacts such as signed URLs | until expiry | 리소스가 살아있는 동안만큼 캐시 |

## Invalidation Rules

- `update` / `delete` 시에는 항상 invalidate한다. 정합성을 TTL에만 의존하지 않는다.
- flush 패턴보다 `del market:${id}` 같은 키 기반 invalidation을 선호한다.
- `markets:active` 같은 리스트 캐시의 경우, write 시 version key를 bump하여 오래된 키가 만료되도록 한다.
- 사용자별 데이터를 공유 키로 캐시하지 않는다. 항상 `user:${id}:`로 네임스페이스를 구분한다.
