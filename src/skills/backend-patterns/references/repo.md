# Repository, Service, Middleware Patterns

Full code samples for the three-tier layering referenced in `SKILL.md`.

## Repository (Supabase)

```typescript
class SupabaseMarketRepository implements MarketRepository {
  async findAll(filters?: MarketFilters): Promise<Market[]> {
    let query = supabase.from('markets').select('*')
    if (filters?.status) query = query.eq('status', filters.status)
    if (filters?.limit) query = query.limit(filters.limit)
    const { data, error } = await query
    if (error) throw new Error(error.message)
    return data
  }
  // findById / create / update / delete follow the same pattern
}
```

## Service Layer

```typescript
class MarketService {
  constructor(private marketRepo: MarketRepository) {}

  async searchMarkets(query: string, limit = 10): Promise<Market[]> {
    const embedding = await generateEmbedding(query)
    const results = await this.vectorSearch(embedding, limit)
    const markets = await this.marketRepo.findByIds(results.map(r => r.id))
    return markets.sort((a, b) => {
      const sa = results.find(r => r.id === a.id)?.score ?? 0
      const sb = results.find(r => r.id === b.id)?.score ?? 0
      return sa - sb
    })
  }

  private async vectorSearch(embedding: number[], limit: number) { /* ... */ }
}
```

## Middleware (Next.js style)

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
if (error) throw new Error('Transaction failed')
```

```sql
CREATE OR REPLACE FUNCTION create_market_with_position(
  market_data jsonb, position_data jsonb
) RETURNS jsonb LANGUAGE plpgsql AS $$
BEGIN
  INSERT INTO markets   VALUES (market_data);
  INSERT INTO positions VALUES (position_data);
  RETURN jsonb_build_object('success', true);
EXCEPTION WHEN OTHERS THEN
  RETURN jsonb_build_object('success', false, 'error', SQLERRM);
END;
$$;
```
