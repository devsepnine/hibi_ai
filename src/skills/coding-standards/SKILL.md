---
name: coding-standards
description: "TS/JS/React/Node coding standards: naming, immutability, error handling, comment rules, code smells, file layout. Use when writing or reviewing code. 코딩 표준, 코드 스타일, 주석 규칙, 코드 리뷰, 클린 코드."
---

# Coding Standards & Best Practices

Universal coding standards. Language/framework specifics delegate to sibling skills.

## Core Principles

- **Readability first, code is the spec**: code is read more than written; names, types, and structure document the *what*, comments carry only the *why*. See Comments below.
- **KISS**: simplest solution that works; no premature optimization.
- **DRY**: extract shared logic; no copy-paste.
- **YAGNI**: don't build for speculative needs; refactor when required.
- **SOLID**: SRP / OCP / LSP / ISP / DIP. One reason to change per module.

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

Files: `Button.tsx` for PascalCase components, `useAuth.ts` for camelCase with a `use` prefix, `formatDate.ts` for camelCase utils, `market.types.ts` for the `.types` suffix.

## TypeScript / JavaScript Patterns

### Immutability: CRITICAL
```typescript
// Always: spread / new object
const updated = { ...user, name: 'New' }
const next = [...items, newItem]

// Never: direct mutation
user.name = 'New'      // BAD
items.push(newItem)    // BAD
```

### Type Safety
Make illegal states unrepresentable. The compiler, not a runtime check, should reject them.

- **Discriminated unions over optional-field bags**: `{ status: 'done'; completedAt: Date } | { status: 'open' }`, not `{ completed: boolean; completedAt?: Date }`.
- **`unknown`, never `any`**, for external data. Narrow by discriminant `switch` > `in` > `typeof`/`instanceof` > type guard. A guard such as `isX` must verify its claim. A lying guard is worse than `as`, because the bug hides behind a name that says it is safe.
- **No `as` to silence the compiler**: parse or narrow instead. `satisfies` checks a literal without widening it.
- **Exhaustive switches**: `default: { const _exhaustive: never = x; throw new Error(...) }` so a new variant fails to compile.
- **Brand primitives whose mix-up is a real bug**: `type UserId = string & { readonly __brand: 'UserId' }`, branded once at the boundary.
- **Derive, don't redeclare**: `z.infer`, `Pick`/`Omit`/`ReturnType`/`Awaited` from the authoritative schema or function before writing a new interface.
- Generics for reusable utilities.

### Boundaries
Parse external input such as request bodies, env, files, and third-party responses into a named domain type once, where it enters. Inside, trust the type: no re-validation, `?.`, or `?? default` guarding a value the type says exists. This covers shape and type checks only. Authorization and output encoding still apply, see `security-review`.

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

## Code Smells: must fix

| Smell | Fix |
|---|---|
| Function > 50 LOC | Extract helpers, one job per function |
| Nesting > 4 levels | Guard clauses / early returns |
| Magic numbers | Named constants |
| Long parameter list, more than 5 | Options object |
| Boolean-flag soup | Split into separate functions; for state, use one union or state machine. A second boolean that must stay in sync is the signal |
| Dead code / commented blocks | Delete |

```typescript
// Guard clauses over deep nesting
if (!user) return
if (!user.isAdmin) return
if (!market?.isActive) return
// ... happy path
```

## Comments: NON-NEGOTIABLE

**Code is the spec.** Names, types, and structure express *what* the code does; a comment exists only for what code cannot say, the *why*: intent, constraints, tradeoffs, invariants, external context. The default is **no comment**, and every comment must earn its place. These rules bind every code edit. Treat a violation like a failing test: fix it before reporting completion.

**Decision procedure, to run before writing any comment:**
1. Would it explain *what* the code does? → Don't write it. Make the code say it instead: rename, extract a function/constant, simplify.
2. Does it carry something code cannot express, such as why this design, which constraint, what tradeoff, or which external fact? → Write it, conclusion first.

**Lead with the conclusion, known as BLUF.** The first line states the point in one sentence; detail follows only if it earns its place. A reader must get the intent from that line alone, without decoding the code under it.

- **Why, not what**: non-obvious decisions, tradeoffs, constraints. No code narration: never restate what the code already says.
- **Summary line first, detail after**: one sentence; add specifics on the following lines only when needed. In block comments, separate them with a blank line; in `//` runs, just continue on the next line. No wall of prose, no multi-line build-up to the point.
- **Cut the noise**: no obvious comments like `// increment i`, no change logs like `// fixed 2026-01-02`, no commented-out code, no emojis.
- **No over-commenting**: comment density is not quality. A comment on every line or block is noise that buries the few comments that matter; if everything is annotated, nothing stands out.
- **Keep it true, in the same edit**: the edit that changes code updates or deletes its comments; a stale comment is worse than no comment because it lies with authority. See *Comment maintenance* below for the refactor cases.
- **Public APIs**: JSDoc/doc comment with params, returns, throws, and an example. Document the *contract*, meaning inputs, outputs, and failure modes, not the implementation. This is an explicit exception to the decision procedure: the contract is not internal *what*. It is the interface a caller cannot see from the call site, so documenting it is required, not optional.
- **Language**: follow the file's existing comment language; never mix two in one file.

```typescript
// Bad — narrates the code and buries the point at the end
// Take retryCount, raise 2 to that power, multiply by 1000, and since that
// grows without bound clamp it with Math.min so the API is not overwhelmed.
const delay = Math.min(1000 * 2 ** retryCount, 30000)

// Good — conclusion first, reason second
// Exponential backoff capped at 30s: protects the API during outages.
const delay = Math.min(1000 * 2 ** retryCount, 30000)
```

When a comment explains *what*, the fix is a refactor, not a better comment:

```typescript
// Bad — the comment props up code that can't speak for itself
// check if the user is allowed to modify this market
if (user.role === 'admin' || (market.ownerId === user.id && !market.closed)) {

// Good — the name carries the spec; no comment needed
if (canModifyMarket(user, market)) {
```

The same rule scales to module/function headers. The first line is the one-sentence contract:

```typescript
/**
 * Fetches a market snapshot, served from cache while it is still fresh.
 *
 * TTL is 5s because the upstream feed batches updates at that interval —
 * a shorter TTL multiplies requests without returning fresher data.
 */
```

### Comment maintenance: the edit that changes code owns its comments

A comment defect is often not written wrong; it is left behind by a later change to the code around it. Run these three checks on every edit that moves, renames, or changes behaviour, before reporting completion. Deleting is a valid outcome for all three: a comment whose reason no longer holds is removed, not reworded.

**1. Move the comment with the code it describes.** Extracting, splitting, inlining, or reordering leaves comments attached to the wrong subject. After an extraction, decide for each line of the original comment which side it now belongs to. A comment that described the whole must not silently become the doc of the part.

```typescript
// Bad — extracting cacheKeyFor left the parent's doc stranded above the child,
// so two doc blocks stack on one declaration and the child's real contract is
// buried under a description of its caller
/** Fetches a market snapshot, served from cache while it is still fresh. */
/** Cache key for a market, namespaced by feed version. */
function cacheKeyFor(marketId: string): string {

// Good — each function carries only its own contract
/** Cache key for a market, namespaced by feed version. */
function cacheKeyFor(marketId: string): string {
```

**2. Sweep past the diff hunk.** A rename or behaviour change invalidates comments the diff never shows, at call sites, in module headers, in sibling files, and in docs. Grep the old name and the old behaviour across the tree: the hunk you edited is where the defect starts, not where it ends.

**3. Verify the *why* you claim.** A *why* comment asserts a fact about the system, and an unverifiable one is a defect even when it reads well, because it sends the next reader hunting a constraint that does not exist. Point at the code path, config, or external source that makes the claim true; if you cannot, state the narrower claim you can support. "Callers may pass a non-ASCII label" is checkable at the signature. "This is the first place user data arrives" is a guess unless you traced every caller.

## Testing: AAA pattern

```typescript
test('returns empty array when no markets match query', () => {
  // Arrange / Act / Assert
})
```
Descriptive names that read as specifications. No `test('works')`. See `tdd-workflow` skill for full TDD loop and 80%+ coverage requirements.

## File Organization

Many small focused files > few large files. Soft 300 LOC, hard 500 LOC per file; see `references/code-thresholds.md`. Organize by feature/domain, not by type.

```
src/
├── app/         # routes / pages
├── components/  # ui, forms, layouts
├── hooks/       # custom hooks (useXxx)
├── lib/         # api clients, utils, constants
├── types/       # shared types
└── styles/
```

## Domain-Specific: see sibling skills

| Concern | Skill |
|---|---|
| React performance: memo, lazy, bundle, RSC | `react-best-practices` |
| React composition / compound components | `composition-patterns` |
| React forms, error boundaries, a11y, animations | `references/react-patterns.md` |
| Full code-review checklist: SOLID, severity, concurrency, cross-platform | `references/review-checklist.md` |
| Common TS patterns: API response, custom hooks, repository, skeleton projects | `references/patterns.md` |
| Code thresholds: file/function LOC, complexity, params, nesting | `references/code-thresholds.md` |
| Zustand global state | `zustand` |
| REST/Next.js API design, validation, DB queries | `backend-patterns` |
| Rust: ownership, errors, async | `rust-best-practices` |
| Security: auth, input validation, secrets | `security-review` |
| Test-first workflow + coverage | `tdd-workflow` |
| Build/type/test verification | `verification-loop` |
| Commit & PR conventions | `commit-rules`, `pull-request` |

**Rule**: do not duplicate framework-specific guidance here. If a topic has a dedicated skill, link to it.
