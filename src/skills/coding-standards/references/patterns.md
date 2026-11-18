# Common Patterns

## API Response Format

```typescript
type ApiResponse<T> =
  | { success: true; data: T; meta?: { total: number; page: number; limit: number } }
  | { success: false; error: string; details?: unknown }
```

## Custom Hooks Pattern

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

## Repository Pattern

```typescript
interface Repository<T> {
  findAll(filters?: Filters): Promise<T[]>
  findById(id: string): Promise<T | null>
  create(data: CreateDto): Promise<T>
  update(id: string, data: UpdateDto): Promise<T>
  delete(id: string): Promise<void>
}
```

## Project templates

For a feature in an existing application, follow the surrounding implementation and project guidance. When choosing a starting point for a new project, check the template's dependencies, supported versions, and maintenance against the requirements.
