---
name: coding-standards
description: Universal coding standards for TypeScript/JavaScript/React/Node — naming, immutability, error handling, code smells, testing, file organization. React form/error-boundary/a11y patterns in references/. 코딩 표준, 코드 스타일, 코드 리뷰, 클린 코드, 폼 검증, 에러 바운더리, 접근성.
keywords: [coding-standards, 코딩표준, 코드스타일, 코드리뷰, clean-code, best-practices, react-patterns, form, error-boundary, a11y, 접근성]
---

# Coding Standards & Best Practices

Universal coding standards. Language/framework specifics delegate to sibling skills.

## Core Principles

- **Readability first** — code is read more than written; self-documenting names beat comments.
- **KISS** — simplest solution that works; no premature optimization.
- **DRY** — extract shared logic; no copy-paste.
- **YAGNI** — don't build for speculative needs; refactor when required.
- **SOLID** — SRP / OCP / LSP / ISP / DIP. One reason to change per module.

## Naming

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

Files: `Button.tsx` (PascalCase components), `useAuth.ts` (camelCase + `use` prefix), `formatDate.ts` (camelCase utils), `market.types.ts` (`.types` suffix).

## TypeScript / JavaScript Patterns

### Immutability (CRITICAL)
```typescript
// Always: spread / new object
const updated = { ...user, name: 'New' }
const next = [...items, newItem]

// Never: direct mutation
user.name = 'New'      // BAD
items.push(newItem)    // BAD
```

### Type Safety
Avoid `any`. Use union literals (`'active' | 'closed'`), `unknown` + narrowing for boundaries, generics for reusable utilities.

### Async / Await
```typescript
// Parallel when independent
const [a, b, c] = await Promise.all([fetchA(), fetchB(), fetchC()])

// Sequential only when one depends on another
```

### Error Handling
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
Never silently swallow with empty `catch {}`. Re-throw with context or handle explicitly.

## Code Smells (must fix)

| Smell | Fix |
|---|---|
| Function > 50 LOC | Extract helpers (one job per function) |
| Nesting > 4 levels | Guard clauses / early returns |
| Magic numbers | Named constants |
| Long parameter list (> 5) | Options object |
| Boolean-flag soup | Split into separate functions |
| Dead code / commented blocks | Delete |

```typescript
// Guard clauses over deep nesting
if (!user) return
if (!user.isAdmin) return
if (!market?.isActive) return
// ... happy path
```

## Comments

Explain **why**, not what. Document non-obvious decisions, tradeoffs, and constraints. JSDoc for public APIs (params, returns, throws, example).

```typescript
// Exponential backoff to avoid overwhelming API during outages
const delay = Math.min(1000 * 2 ** retryCount, 30000)
```

## Testing (AAA pattern)

```typescript
test('returns empty array when no markets match query', () => {
  // Arrange / Act / Assert
})
```
Descriptive names that read as specifications. No `test('works')`. See `tdd-workflow` skill for full TDD loop and 80%+ coverage requirements.

## File Organization

Many small focused files > few large files. Soft 300 LOC, hard 500 LOC per file (see `references/code-thresholds.md`). Organize by feature/domain, not by type.

```
src/
├── app/         # routes / pages
├── components/  # ui, forms, layouts
├── hooks/       # custom hooks (useXxx)
├── lib/         # api clients, utils, constants
├── types/       # shared types
└── styles/
```

## Domain-Specific — see sibling skills

| Concern | Skill |
|---|---|
| React performance (memo, lazy, bundle, RSC) | `react-best-practices` |
| React composition / compound components | `composition-patterns` |
| React forms, error boundaries, a11y, animations | `references/react-patterns.md` |
| Full code-review checklist (SOLID, severity, concurrency, cross-platform) | `references/review-checklist.md` |
| Common TS patterns (API response, custom hooks, repository, skeleton projects) | `references/patterns.md` |
| Code thresholds (file/function LOC, complexity, params, nesting) | `references/code-thresholds.md` |
| Zustand global state | `zustand` |
| REST/Next.js API design, validation, DB queries | `backend-patterns` |
| Rust (ownership, errors, async) | `rust-best-practices` |
| Security (auth, input validation, secrets) | `security-review` |
| Test-first workflow + coverage | `tdd-workflow` |
| Build/type/test verification | `verification-loop` |
| Commit & PR conventions | `commit-rules`, `pull-request` |

**Rule**: do not duplicate framework-specific guidance here. If a topic has a dedicated skill, link to it.
