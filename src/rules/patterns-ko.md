# Common Patterns

## API 응답 형식

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  meta?: {
    total: number
    page: number
    limit: number
  }
}
```

## Custom Hooks 패턴

```typescript
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const handler = setTimeout(() => setDebouncedValue(value), delay)
    return () => clearTimeout(handler)
  }, [value, delay])

  return debouncedValue
}
```

## Repository 패턴

```typescript
interface Repository<T> {
  findAll(filters?: Filters): Promise<T[]>
  findById(id: string): Promise<T | null>
  create(data: CreateDto): Promise<T>
  update(id: string, data: UpdateDto): Promise<T>
  delete(id: string): Promise<void>
}
```

## Skeleton 프로젝트

신규 기능을 구현할 때:
1. 검증된 skeleton 프로젝트를 검색한다
2. 병렬 에이전트로 옵션을 평가한다:
   - 보안 평가
   - 확장성 분석
   - 관련성 점수
   - 구현 계획
3. 가장 적합한 후보를 베이스로 클론한다
4. 검증된 구조 안에서 반복한다
