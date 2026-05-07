# Coding Style

## 불변성 (CRITICAL)

항상 새 객체를 만들고 절대 변경하지 않는다.

```javascript
// WRONG: Mutation
function updateUser(user, name) {
  user.name = name  // MUTATION!
  return user
}

// CORRECT: Immutability
function updateUser(user, name) {
  return {
    ...user,
    name
  }
}
```

## 파일 구성

작은 파일 다수가 큰 파일 소수보다 낫다 (MANY SMALL FILES > FEW LARGE FILES):
- 높은 응집, 낮은 결합
- 일반적으로 200-300 LOC, 최대 500 (`code-thresholds.md` 참조)
- 작은 파일보다 작은 함수를 우선한다
- 큰 컴포넌트에서 유틸리티를 추출한다
- 타입이 아닌 기능/도메인 단위로 구성한다

## 에러 처리

항상 에러를 빠짐없이 처리한다.

```typescript
try {
  const result = await riskyOperation()
  return result
} catch (error) {
  console.error('Operation failed:', error)
  throw new Error('Detailed user-friendly message')
}
```

## 입력 검증

항상 사용자 입력을 검증한다.

```typescript
import { z } from 'zod'

const schema = z.object({
  email: z.string().email(),
  age: z.number().int().min(0).max(150)
})

const validated = schema.parse(input)
```

## 코드 품질 체크리스트

작업 완료 표시 전에 확인:
- [ ] 코드가 가독성 있고 잘 명명되었는가
- [ ] 함수가 작은가 (≤50 LOC soft, ≤80 hard — `code-thresholds.md` 참조)
- [ ] 파일이 집중되어 있는가 (≤300 LOC soft, ≤500 hard)
- [ ] 깊은 중첩이 없는가 (>4 levels)
- [ ] 적절한 에러 처리가 있는가
- [ ] console.log 문이 없는가
- [ ] 하드코딩된 값이 없는가
- [ ] 변경(mutation)이 없는가 (불변 패턴 사용)
