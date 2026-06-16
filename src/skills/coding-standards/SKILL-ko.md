---
name: coding-standards
description: Universal coding standards for TypeScript/JavaScript/React/Node — naming, immutability, error handling, code smells, testing, file organization. React form/error-boundary/a11y patterns in references/. 코딩 표준, 코드 스타일, 코드 리뷰, 클린 코드, 폼 검증, 에러 바운더리, 접근성.
keywords: [coding-standards, 코딩표준, 코드스타일, 코드리뷰, clean-code, best-practices, react-patterns, form, error-boundary, a11y, 접근성]
---

# 코딩 표준 & 모범 사례

범용 코딩 표준. 언어/프레임워크 특화 사항은 자매 skill에 위임한다.

## 핵심 원칙

- **가독성 우선** — 코드는 작성보다 읽히는 일이 많다; 자기 문서화 이름이 주석을 이긴다.
- **KISS** — 동작하는 가장 단순한 솔루션; 성급한 최적화 금지.
- **DRY** — 공통 로직 추출; 복사-붙여넣기 금지.
- **YAGNI** — 추측성 요구를 위해 만들지 말 것; 필요해질 때 리팩토링한다.
- **SOLID** — SRP / OCP / LSP / ISP / DIP. 모듈당 변경 이유는 하나.

## 명명

```typescript
// Variables: descriptive, intent-revealing
const marketSearchQuery = 'election'   // not q
const isUserAuthenticated = true       // not flag

// Functions: verb-noun
async function fetchMarketData(id: string) {}
function isValidEmail(email: string): boolean {}

// Constants: SCREAMING_SNAKE for magic values
const MAX_RETRIES = 3
const DEBOUNCE_DELAY_MS = 500
```

파일: `Button.tsx` (PascalCase 컴포넌트), `useAuth.ts` (camelCase + `use` 접두사), `formatDate.ts` (camelCase 유틸), `market.types.ts` (`.types` 접미사).

## TypeScript / JavaScript 패턴

### 불변성 (CRITICAL)
```typescript
// Always: spread / new object
const updated = { ...user, name: 'New' }
const next = [...items, newItem]

// Never: direct mutation
user.name = 'New'      // BAD
items.push(newItem)    // BAD
```

### 타입 안전성
`any` 회피. 유니언 리터럴 (`'active' | 'closed'`), 경계에는 `unknown` + 내로잉, 재사용 유틸리티에는 제네릭을 사용한다.

### Async / Await
```typescript
// Parallel when independent
const [a, b, c] = await Promise.all([fetchA(), fetchB(), fetchC()])

// Sequential only when one depends on another
```

### 에러 처리
```typescript
try {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return await res.json()
} catch (error) {
  console.error('Fetch failed:', error)
  throw new Error('Failed to fetch data')  // user-facing, no leak
}
```
빈 `catch {}`로 조용히 삼키지 말 것. 컨텍스트와 함께 다시 throw하거나 명시적으로 처리한다.

## 코드 스멜 (반드시 수정)

| Smell | Fix |
|---|---|
| 함수 > 50 LOC | helper 추출 (함수당 한 가지 일) |
| 중첩 > 4단계 | guard clause / 조기 반환 |
| 매직 넘버 | 명명된 상수 |
| 긴 파라미터 목록 (> 5) | options 객체 |
| boolean 플래그 수렁 | 별도 함수로 분리 |
| 죽은 코드 / 주석 처리된 블록 | 삭제 |

```typescript
// Guard clauses over deep nesting
if (!user) return
if (!user.isAdmin) return
if (!market?.isActive) return
// ... happy path
```

## 주석

**왜**를 설명하라, 무엇을이 아니라. 비자명한 결정, 트레이드오프, 제약사항을 문서화한다. 공개 API에는 JSDoc (params, returns, throws, example).

```typescript
// Exponential backoff to avoid overwhelming API during outages
const delay = Math.min(1000 * 2 ** retryCount, 30000)
```

## 테스팅 (AAA 패턴)

```typescript
test('returns empty array when no markets match query', () => {
  // Arrange / Act / Assert
})
```
명세처럼 읽히는 서술적 이름. `test('works')` 같은 건 금지. 전체 TDD 루프와 80%+ 커버리지 요구사항은 `tdd-workflow` skill 참조.

## 파일 조직화

작고 집중된 다수의 파일 > 큰 파일 소수. 파일당 soft 300 LOC, hard 500 LOC (`references/code-thresholds.md` 참조). 타입이 아닌 feature/domain 으로 조직한다.

```
src/
├── app/         # routes / pages
├── components/  # ui, forms, layouts
├── hooks/       # custom hooks (useXxx)
├── lib/         # api clients, utils, constants
├── types/       # shared types
└── styles/
```

## 도메인 특화 — 자매 skill 참조

| Concern | Skill |
|---|---|
| React 성능 (memo, lazy, bundle, RSC) | `react-best-practices` |
| React 컴포지션 / compound components | `composition-patterns` |
| React 폼, 에러 바운더리, a11y, 애니메이션 | `references/react-patterns.md` |
| 전체 코드 리뷰 체크리스트 (SOLID, severity, concurrency, cross-platform) | `references/review-checklist.md` |
| 공통 TS 패턴 (API 응답, 커스텀 훅, repository, skeleton 프로젝트) | `references/patterns.md` |
| 코드 임계값 (파일/함수 LOC, 복잡도, 파라미터, 중첩) | `references/code-thresholds.md` |
| Zustand 전역 상태 | `zustand` |
| REST/Next.js API 설계, 검증, DB 쿼리 | `backend-patterns` |
| Rust (소유권, 에러, async) | `rust-best-practices` |
| 보안 (인증, 입력 검증, 시크릿) | `security-review` |
| 테스트 우선 워크플로우 + 커버리지 | `tdd-workflow` |
| 빌드/타입/테스트 검증 | `verification-loop` |
| 커밋 & PR 컨벤션 | `commit-rules`, `pull-request` |

**규칙**: 여기에 프레임워크 특화 가이드를 중복하지 말 것. 전용 skill이 있는 주제면 링크한다.
