# Common Patterns

## API 응답 형식

```typescript
type ApiResponse<T> =
  | { success: true; data: T; meta?: { total: number; page: number; limit: number } }
  | { success: false; error: string; details?: unknown }
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

## 프로젝트 템플릿

기존 앱에 기능을 추가할 때는 인접한 구현과 프로젝트의 개발 지침을 따른다. 새 프로젝트의 시작 구조를 선택하는 작업이라면 템플릿의 의존성, 지원 버전, 유지관리 상태가 요구사항에 맞는지 확인한다.
