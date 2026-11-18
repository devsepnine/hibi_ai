# Repository, Service, Middleware Patterns

`SKILL.md`에서 참조하는 3-tier 레이어링에 대한 전체 코드 샘플이다.

## Repository, Supabase

```typescript
class SupabaseMarketQueries implements Pick<MarketRepository, 'findAll' | 'findByIds'> {
  async findAll(filters?: MarketFilters): Promise<Market[]> {
    let query = supabase.from('markets').select('*')
    if (filters?.status) query = query.eq('status', filters.status)
    if (filters?.limit) query = query.limit(filters.limit)
    const { data, error } = await query
    if (error) throw new Error(error.message)
    return data
  }

  async findByIds(ids: readonly string[]): Promise<readonly Market[]> {
    if (ids.length === 0) return []
    const { data, error } = await supabase.from('markets').select('*').in('id', ids)
    if (error) throw new Error(error.message)
    return data
  }
}
```

## Service Layer

이 서비스는 프로젝트의 임베딩 함수와 벡터 검색 함수를 사용한다. 이 예시에서 검색 점수는 거리이며 가장 가까운 시장을 먼저 반환한다.

```typescript
class MarketService {
  constructor(private marketRepo: Pick<MarketRepository, 'findByIds'>) {}

  async searchMarkets(query: string, limit = 10): Promise<Market[]> {
    const embedding = await generateEmbedding(query)
    const results = await vectorSearch(embedding, limit)
    const markets = await this.marketRepo.findByIds(results.map(r => r.id))
    return [...markets].sort((a, b) => {
      const sa = results.find(r => r.id === a.id)?.score ?? 0
      const sb = results.find(r => r.id === b.id)?.score ?? 0
      return sa - sb
    })
  }
}
```

## Middleware, Next.js style

```typescript
export function withAuth(handler: NextApiHandler): NextApiHandler {
  return async (req, res) => {
    const token = req.headers.authorization?.replace('Bearer ', '')
    if (!token) return res.status(401).json({ error: 'Unauthorized' })
    try {
      req.user = await verifyToken(token)
      return handler(req, res)
    } catch {
      return res.status(401).json({ error: 'Invalid token' })
    }
  }
}

// Compose: withRateLimit(withAuth(handler))
```

## Transaction via Supabase RPC

```typescript
const { data, error } = await supabase.rpc('create_market_with_position', {
  market_data: marketData,
  position_data: positionData,
})
if (error) throw error
```

```sql
CREATE OR REPLACE FUNCTION create_market_with_position(
  market_data jsonb, position_data jsonb
) RETURNS jsonb LANGUAGE plpgsql AS $$
BEGIN
  INSERT INTO markets   VALUES (market_data);
  INSERT INTO positions VALUES (position_data);
  RETURN jsonb_build_object('success', true);
END;
$$;
```
