# React Best Practices

**Version 1.0.0**  
Vercel Engineering  
January 2026

> **Note:**  
> 이 문서는 주로 에이전트와 LLM이 React·Next.js 코드베이스를 유지보수, 생성, 리팩토링할 때 따르는 지침이다. 사람이 읽어도 유용하지만, AI 보조 워크플로우의 자동화·일관성에 맞춰 최적화되어 있다.

---

## Abstract

React·Next.js 애플리케이션을 위한 종합적인 성능 최적화 가이드이다. AI 에이전트와 LLM을 위해 설계되었다. 8개 카테고리에 걸쳐 40개 이상의 룰을 영향도 우선순위로 정리한다 — 핵심(워터폴 제거, 번들 크기 축소)부터 점진적 개선(고급 패턴)까지 다룬다. 각 룰에는 자세한 설명, 부정확/정확 구현을 비교하는 실제 예시, 그리고 자동 리팩토링·코드 생성을 안내하는 구체적인 영향 지표가 포함되어 있다.

---

## Table of Contents

1. [Eliminating Waterfalls](#1-eliminating-waterfalls) — **CRITICAL**
   - 1.1 [Check Cheap Conditions Before Async Flags](#11-check-cheap-conditions-before-async-flags)
   - 1.2 [Defer Await Until Needed](#12-defer-await-until-needed)
   - 1.3 [Dependency-Based Parallelization](#13-dependency-based-parallelization)
   - 1.4 [Prevent Waterfall Chains in API Routes](#14-prevent-waterfall-chains-in-api-routes)
   - 1.5 [Promise.all() for Independent Operations](#15-promiseall-for-independent-operations)
   - 1.6 [Strategic Suspense Boundaries](#16-strategic-suspense-boundaries)
2. [Bundle Size Optimization](#2-bundle-size-optimization) — **CRITICAL**
   - 2.1 [Avoid Barrel File Imports](#21-avoid-barrel-file-imports)
   - 2.2 [Conditional Module Loading](#22-conditional-module-loading)
   - 2.3 [Defer Non-Critical Third-Party Libraries](#23-defer-non-critical-third-party-libraries)
   - 2.4 [Dynamic Imports for Heavy Components](#24-dynamic-imports-for-heavy-components)
   - 2.5 [Preload Based on User Intent](#25-preload-based-on-user-intent)
3. [Server-Side Performance](#3-server-side-performance) — **HIGH**
   - 3.1 [Authenticate Server Actions Like API Routes](#31-authenticate-server-actions-like-api-routes)
   - 3.2 [Avoid Duplicate Serialization in RSC Props](#32-avoid-duplicate-serialization-in-rsc-props)
   - 3.3 [Cross-Request LRU Caching](#33-cross-request-lru-caching)
   - 3.4 [Hoist Static I/O to Module Level](#34-hoist-static-io-to-module-level)
   - 3.5 [Minimize Serialization at RSC Boundaries](#35-minimize-serialization-at-rsc-boundaries)
   - 3.6 [Parallel Data Fetching with Component Composition](#36-parallel-data-fetching-with-component-composition)
   - 3.7 [Parallel Nested Data Fetching](#37-parallel-nested-data-fetching)
   - 3.8 [Per-Request Deduplication with React.cache()](#38-per-request-deduplication-with-reactcache)
   - 3.9 [Use after() for Non-Blocking Operations](#39-use-after-for-non-blocking-operations)
4. [Client-Side Data Fetching](#4-client-side-data-fetching) — **MEDIUM-HIGH**
   - 4.1 [Deduplicate Global Event Listeners](#41-deduplicate-global-event-listeners)
   - 4.2 [Use Passive Event Listeners for Scrolling Performance](#42-use-passive-event-listeners-for-scrolling-performance)
   - 4.3 [Use SWR for Automatic Deduplication](#43-use-swr-for-automatic-deduplication)
   - 4.4 [Version and Minimize localStorage Data](#44-version-and-minimize-localstorage-data)
5. [Re-render Optimization](#5-re-render-optimization) — **MEDIUM**
   - 5.1 [Calculate Derived State During Rendering](#51-calculate-derived-state-during-rendering)
   - 5.2 [Defer State Reads to Usage Point](#52-defer-state-reads-to-usage-point)
   - 5.3 [Do not wrap a simple expression with a primitive result type in useMemo](#53-do-not-wrap-a-simple-expression-with-a-primitive-result-type-in-usememo)
   - 5.4 [Don't Define Components Inside Components](#54-dont-define-components-inside-components)
   - 5.5 [Extract Default Non-primitive Parameter Value from Memoized Component to Constant](#55-extract-default-non-primitive-parameter-value-from-memoized-component-to-constant)
   - 5.6 [Extract to Memoized Components](#56-extract-to-memoized-components)
   - 5.7 [Narrow Effect Dependencies](#57-narrow-effect-dependencies)
   - 5.8 [Put Interaction Logic in Event Handlers](#58-put-interaction-logic-in-event-handlers)
   - 5.9 [Split Combined Hook Computations](#59-split-combined-hook-computations)
   - 5.10 [Subscribe to Derived State](#510-subscribe-to-derived-state)
   - 5.11 [Use Functional setState Updates](#511-use-functional-setstate-updates)
   - 5.12 [Use Lazy State Initialization](#512-use-lazy-state-initialization)
   - 5.13 [Use Transitions for Non-Urgent Updates](#513-use-transitions-for-non-urgent-updates)
   - 5.14 [Use useDeferredValue for Expensive Derived Renders](#514-use-usedeferredvalue-for-expensive-derived-renders)
   - 5.15 [Use useRef for Transient Values](#515-use-useref-for-transient-values)
6. [Rendering Performance](#6-rendering-performance) — **MEDIUM**
   - 6.1 [Animate SVG Wrapper Instead of SVG Element](#61-animate-svg-wrapper-instead-of-svg-element)
   - 6.2 [CSS content-visibility for Long Lists](#62-css-content-visibility-for-long-lists)
   - 6.3 [Hoist Static JSX Elements](#63-hoist-static-jsx-elements)
   - 6.4 [Optimize SVG Precision](#64-optimize-svg-precision)
   - 6.5 [Prevent Hydration Mismatch Without Flickering](#65-prevent-hydration-mismatch-without-flickering)
   - 6.6 [Suppress Expected Hydration Mismatches](#66-suppress-expected-hydration-mismatches)
   - 6.7 [Use Activity Component for Show/Hide](#67-use-activity-component-for-showhide)
   - 6.8 [Use defer or async on Script Tags](#68-use-defer-or-async-on-script-tags)
   - 6.9 [Use Explicit Conditional Rendering](#69-use-explicit-conditional-rendering)
   - 6.10 [Use React DOM Resource Hints](#610-use-react-dom-resource-hints)
   - 6.11 [Use useTransition Over Manual Loading States](#611-use-usetransition-over-manual-loading-states)
7. [JavaScript Performance](#7-javascript-performance) — **LOW-MEDIUM**
   - 7.1 [Avoid Layout Thrashing](#71-avoid-layout-thrashing)
   - 7.2 [Build Index Maps for Repeated Lookups](#72-build-index-maps-for-repeated-lookups)
   - 7.3 [Cache Property Access in Loops](#73-cache-property-access-in-loops)
   - 7.4 [Cache Repeated Function Calls](#74-cache-repeated-function-calls)
   - 7.5 [Cache Storage API Calls](#75-cache-storage-api-calls)
   - 7.6 [Combine Multiple Array Iterations](#76-combine-multiple-array-iterations)
   - 7.7 [Defer Non-Critical Work with requestIdleCallback](#77-defer-non-critical-work-with-requestidlecallback)
   - 7.8 [Early Length Check for Array Comparisons](#78-early-length-check-for-array-comparisons)
   - 7.9 [Early Return from Functions](#79-early-return-from-functions)
   - 7.10 [Hoist RegExp Creation](#710-hoist-regexp-creation)
   - 7.11 [Use flatMap to Map and Filter in One Pass](#711-use-flatmap-to-map-and-filter-in-one-pass)
   - 7.12 [Use Loop for Min/Max Instead of Sort](#712-use-loop-for-minmax-instead-of-sort)
   - 7.13 [Use Set/Map for O(1) Lookups](#713-use-setmap-for-o1-lookups)
   - 7.14 [Use toSorted() Instead of sort() for Immutability](#714-use-tosorted-instead-of-sort-for-immutability)
8. [Advanced Patterns](#8-advanced-patterns) — **LOW**
   - 8.1 [Initialize App Once, Not Per Mount](#81-initialize-app-once-not-per-mount)
   - 8.2 [Store Event Handlers in Refs](#82-store-event-handlers-in-refs)
   - 8.3 [useEffectEvent for Stable Callback Refs](#83-useeffectevent-for-stable-callback-refs)

---

## 1. Eliminating Waterfalls

**Impact: CRITICAL**

워터폴은 성능을 가장 크게 저해하는 요인이다. 순차 await 하나마다 풀 네트워크 지연이 누적된다. 이를 제거하면 가장 큰 성능 향상을 얻는다.

### 1.1 Check Cheap Conditions Before Async Flags

**Impact: HIGH (avoids unnecessary async work when a synchronous guard already fails)**

플래그나 원격 값을 위한 `await`이 들어가는 분기에서 **저비용 동기** 조건(local props, request metadata, 이미 로드된 state)도 함께 요구된다면, 동기 조건을 **먼저** 평가한다. 그렇지 않으면 합성 조건이 결코 참이 될 수 없는 경우에도 async 호출 비용을 지불한다.

이는 [Defer Await Until Needed](./async-defer-await.md)를 `flag && cheapCondition` 형태에 특화한 변형이다.

**Incorrect:**

```typescript
const someFlag = await getFlag()

if (someFlag && someCondition) {
  // ...
}
```

**Correct:**

```typescript
if (someCondition) {
  const someFlag = await getFlag()
  if (someFlag) {
    // ...
  }
}
```

`getFlag`가 네트워크, feature-flag 서비스, `React.cache` / DB 작업을 호출할 때 의미가 크다. `someCondition`이 false일 때 호출 자체를 건너뛰어 cold path 비용을 제거한다.

`someCondition`이 비싸거나, 플래그에 의존하거나, 부수효과를 정해진 순서로 실행해야 한다면 원래 순서를 유지한다.

### 1.2 Defer Await Until Needed

**Impact: HIGH (avoids blocking unused code paths)**

`await` 연산을 실제로 사용하는 분기 안으로 옮겨서, 해당 데이터가 필요 없는 코드 경로가 막히지 않도록 한다.

**Incorrect: blocks both branches**

```typescript
async function handleRequest(userId: string, skipProcessing: boolean) {
  const userData = await fetchUserData(userId)
  
  if (skipProcessing) {
    // Returns immediately but still waited for userData
    return { skipped: true }
  }
  
  // Only this branch uses userData
  return processUserData(userData)
}
```

**Correct: only blocks when needed**

```typescript
async function handleRequest(userId: string, skipProcessing: boolean) {
  if (skipProcessing) {
    // Returns immediately without waiting
    return { skipped: true }
  }
  
  // Fetch only when needed
  const userData = await fetchUserData(userId)
  return processUserData(userData)
}
```

**Another example: early return optimization**

```typescript
// Incorrect: always fetches permissions
async function updateResource(resourceId: string, userId: string) {
  const permissions = await fetchPermissions(userId)
  const resource = await getResource(resourceId)
  
  if (!resource) {
    return { error: 'Not found' }
  }
  
  if (!permissions.canEdit) {
    return { error: 'Forbidden' }
  }
  
  return await updateResourceData(resource, permissions)
}

// Correct: fetches only when needed
async function updateResource(resourceId: string, userId: string) {
  const resource = await getResource(resourceId)
  
  if (!resource) {
    return { error: 'Not found' }
  }
  
  const permissions = await fetchPermissions(userId)
  
  if (!permissions.canEdit) {
    return { error: 'Forbidden' }
  }
  
  return await updateResourceData(resource, permissions)
}
```

스킵되는 분기가 자주 선택되거나 지연된 작업이 비쌀수록 이 최적화가 더 가치 있다.

`flag && someCondition`처럼 `await getFlag()`와 저비용 동기 가드가 결합된 경우는 [Check Cheap Conditions Before Async Flags](./async-cheap-condition-before-await.md)를 참고한다.

### 1.3 Dependency-Based Parallelization

**Impact: CRITICAL (2-10× improvement)**

부분 의존성이 있는 작업들에 대해서는 `better-all`을 사용해 병렬성을 극대화한다. 각 작업을 가능한 가장 이른 시점에 자동으로 시작한다.

**Incorrect: profile waits for config unnecessarily**

```typescript
const [user, config] = await Promise.all([
  fetchUser(),
  fetchConfig()
])
const profile = await fetchProfile(user.id)
```

**Correct: config and profile run in parallel**

```typescript
import { all } from 'better-all'

const { user, config, profile } = await all({
  async user() { return fetchUser() },
  async config() { return fetchConfig() },
  async profile() {
    return fetchProfile((await this.$.user).id)
  }
})
```

**Alternative without extra dependencies:**

```typescript
const userPromise = fetchUser()
const profilePromise = userPromise.then(user => fetchProfile(user.id))

const [user, config, profile] = await Promise.all([
  userPromise,
  fetchConfig(),
  profilePromise
])
```

추가 의존성 없이 모든 promise를 먼저 만들고 마지막에 `Promise.all()`을 호출하는 방법도 있다.

Reference: [https://github.com/shuding/better-all](https://github.com/shuding/better-all)

### 1.4 Prevent Waterfall Chains in API Routes

**Impact: CRITICAL (2-10× improvement)**

API 라우트와 Server Action에서는 독립적인 작업은 await을 미루더라도 즉시 시작한다.

**Incorrect: config waits for auth, data waits for both**

```typescript
export async function GET(request: Request) {
  const session = await auth()
  const config = await fetchConfig()
  const data = await fetchData(session.user.id)
  return Response.json({ data, config })
}
```

**Correct: auth and config start immediately**

```typescript
export async function GET(request: Request) {
  const sessionPromise = auth()
  const configPromise = fetchConfig()
  const session = await sessionPromise
  const [config, data] = await Promise.all([
    configPromise,
    fetchData(session.user.id)
  ])
  return Response.json({ data, config })
}
```

더 복잡한 의존성 체인을 가진 작업이라면 `better-all`을 사용해 자동으로 병렬성을 극대화한다 (Dependency-Based Parallelization 참고).

### 1.5 Promise.all() for Independent Operations

**Impact: CRITICAL (2-10× improvement)**

비동기 작업들 사이에 의존성이 없다면 `Promise.all()`로 동시에 실행한다.

**Incorrect: sequential execution, 3 round trips**

```typescript
const user = await fetchUser()
const posts = await fetchPosts()
const comments = await fetchComments()
```

**Correct: parallel execution, 1 round trip**

```typescript
const [user, posts, comments] = await Promise.all([
  fetchUser(),
  fetchPosts(),
  fetchComments()
])
```

### 1.6 Strategic Suspense Boundaries

**Impact: HIGH (faster initial paint)**

async 컴포넌트에서 JSX를 반환하기 전에 데이터를 await하지 말고, Suspense 경계를 사용해 데이터가 로드되는 동안 외곽 UI를 먼저 보여준다.

**Incorrect: wrapper blocked by data fetching**

```tsx
async function Page() {
  const data = await fetchData() // Blocks entire page
  
  return (
    <div>
      <div>Sidebar</div>
      <div>Header</div>
      <div>
        <DataDisplay data={data} />
      </div>
      <div>Footer</div>
    </div>
  )
}
```

가운데 영역만 데이터가 필요한데도 전체 레이아웃이 데이터를 기다린다.

**Correct: wrapper shows immediately, data streams in**

```tsx
function Page() {
  return (
    <div>
      <div>Sidebar</div>
      <div>Header</div>
      <div>
        <Suspense fallback={<Skeleton />}>
          <DataDisplay />
        </Suspense>
      </div>
      <div>Footer</div>
    </div>
  )
}

async function DataDisplay() {
  const data = await fetchData() // Only blocks this component
  return <div>{data.content}</div>
}
```

Sidebar, Header, Footer는 즉시 렌더링되고 DataDisplay만 데이터를 기다린다.

**Alternative: share promise across components**

```tsx
function Page() {
  // Start fetch immediately, but don't await
  const dataPromise = fetchData()
  
  return (
    <div>
      <div>Sidebar</div>
      <div>Header</div>
      <Suspense fallback={<Skeleton />}>
        <DataDisplay dataPromise={dataPromise} />
        <DataSummary dataPromise={dataPromise} />
      </Suspense>
      <div>Footer</div>
    </div>
  )
}

function DataDisplay({ dataPromise }: { dataPromise: Promise<Data> }) {
  const data = use(dataPromise) // Unwraps the promise
  return <div>{data.content}</div>
}

function DataSummary({ dataPromise }: { dataPromise: Promise<Data> }) {
  const data = use(dataPromise) // Reuses the same promise
  return <div>{data.summary}</div>
}
```

두 컴포넌트가 같은 promise를 공유하므로 fetch는 한 번만 일어난다. 레이아웃은 즉시 그려지고 두 컴포넌트는 함께 대기한다.

**When NOT to use this pattern:**

- 레이아웃 결정에 필요한 핵심 데이터(위치에 영향)

- SEO에 중요한 above-the-fold 콘텐츠

- suspense 오버헤드가 무시할 만큼 작고 빠른 쿼리

- 레이아웃 시프트(loading → content jump)를 피하고 싶을 때

**Trade-off:** 빠른 초기 페인트 vs 잠재적 레이아웃 시프트. UX 우선순위에 따라 선택한다.

---

## 2. Bundle Size Optimization

**Impact: CRITICAL**

초기 번들 크기를 줄이면 Time to Interactive와 Largest Contentful Paint가 개선된다.

### 2.1 Avoid Barrel File Imports

**Impact: CRITICAL (200-800ms import cost, slow builds)**

수천 개의 사용되지 않는 모듈을 로드하지 않도록 barrel 파일 대신 소스 파일에서 직접 import한다. **Barrel 파일**은 여러 모듈을 다시 export하는 진입점 파일이다 (예: `export * from './module'`을 하는 `index.js`).

인기 있는 아이콘·컴포넌트 라이브러리는 진입점 파일에 **최대 10,000개의 re-export**가 들어 있을 수 있다. 다수의 React 패키지는 **import만으로 200-800ms가 소요되어** 개발 속도와 프로덕션 cold start에 모두 영향을 준다.

**Why tree-shaking doesn't help:** 라이브러리가 external로 표시되면(번들에 포함되지 않으면) 번들러가 최적화를 할 수 없다. tree-shaking을 위해 번들에 포함시키면 전체 모듈 그래프 분석 때문에 빌드가 크게 느려진다.

**Incorrect: imports entire library**

```tsx
import { Check, X, Menu } from 'lucide-react'
// Loads 1,583 modules, takes ~2.8s extra in dev
// Runtime cost: 200-800ms on every cold start

import { Button, TextField } from '@mui/material'
// Loads 2,225 modules, takes ~4.2s extra in dev
```

**Correct - Next.js 13.5+ (recommended):**

```tsx
// Keep the standard imports - Next.js transforms them to direct imports
import { Check, X, Menu } from 'lucide-react'
// Full TypeScript support, no manual path wrangling
```

이 방식이 권장된다. TypeScript 타입 안전성과 에디터 자동완성을 유지하면서 barrel import 비용을 제거할 수 있기 때문이다.

**Correct - Direct imports (non-Next.js projects):**

```tsx
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
// Loads only what you use
```

> **TypeScript warning:** 일부 라이브러리(특히 `lucide-react`)는 deep import 경로에 대해 `.d.ts`를 제공하지 않는다. `lucide-react/dist/esm/icons/check`를 import하면 implicit `any`로 해석되어 `strict`나 `noImplicitAny` 환경에서 에러가 발생한다. 가능하면 `optimizePackageImports`를 우선 사용하고, direct import 전에 라이브러리가 subpath 타입을 export하는지 확인한다.

이러한 최적화로 dev boot 15-70% 단축, 빌드 28% 단축, cold start 40% 단축, HMR 속도가 크게 향상된다.

자주 영향받는 라이브러리: `lucide-react`, `@mui/material`, `@mui/icons-material`, `@tabler/icons-react`, `react-icons`, `@headlessui/react`, `@radix-ui/react-*`, `lodash`, `ramda`, `date-fns`, `rxjs`, `react-use`.

Reference: [https://vercel.com/blog/how-we-optimized-package-imports-in-next-js](https://vercel.com/blog/how-we-optimized-package-imports-in-next-js)

### 2.2 Conditional Module Loading

**Impact: HIGH (loads large data only when needed)**

큰 데이터나 모듈은 해당 기능이 활성화된 경우에만 로드한다.

**Example: lazy-load animation frames**

```tsx
function AnimationPlayer({ enabled, setEnabled }: { enabled: boolean; setEnabled: React.Dispatch<React.SetStateAction<boolean>> }) {
  const [frames, setFrames] = useState<Frame[] | null>(null)

  useEffect(() => {
    if (enabled && !frames && typeof window !== 'undefined') {
      import('./animation-frames.js')
        .then(mod => setFrames(mod.frames))
        .catch(() => setEnabled(false))
    }
  }, [enabled, frames, setEnabled])

  if (!frames) return <Skeleton />
  return <Canvas frames={frames} />
}
```

`typeof window !== 'undefined'` 체크는 SSR에서 이 모듈이 번들에 포함되지 않게 막아 서버 번들 크기와 빌드 속도를 최적화한다.

### 2.3 Defer Non-Critical Third-Party Libraries

**Impact: MEDIUM (loads after hydration)**

Analytics, logging, error tracking는 사용자 상호작용을 막지 않는다. hydration 이후에 로드한다.

**Incorrect: blocks initial bundle**

```tsx
import { Analytics } from '@vercel/analytics/react'

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  )
}
```

**Correct: loads after hydration**

```tsx
import dynamic from 'next/dynamic'

const Analytics = dynamic(
  () => import('@vercel/analytics/react').then(m => m.Analytics),
  { ssr: false }
)

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  )
}
```

### 2.4 Dynamic Imports for Heavy Components

**Impact: CRITICAL (directly affects TTI and LCP)**

초기 렌더링에 필요 없는 큰 컴포넌트는 `next/dynamic`으로 lazy-load한다.

**Incorrect: Monaco bundles with main chunk ~300KB**

```tsx
import { MonacoEditor } from './monaco-editor'

function CodePanel({ code }: { code: string }) {
  return <MonacoEditor value={code} />
}
```

**Correct: Monaco loads on demand**

```tsx
import dynamic from 'next/dynamic'

const MonacoEditor = dynamic(
  () => import('./monaco-editor').then(m => m.MonacoEditor),
  { ssr: false }
)

function CodePanel({ code }: { code: string }) {
  return <MonacoEditor value={code} />
}
```

### 2.5 Preload Based on User Intent

**Impact: MEDIUM (reduces perceived latency)**

사용자가 사용할 가능성이 높은 시점에 무거운 번들을 미리 로드해 체감 지연을 줄인다.

**Example: preload on hover/focus**

```tsx
function EditorButton({ onClick }: { onClick: () => void }) {
  const preload = () => {
    if (typeof window !== 'undefined') {
      void import('./monaco-editor')
    }
  }

  return (
    <button
      onMouseEnter={preload}
      onFocus={preload}
      onClick={onClick}
    >
      Open Editor
    </button>
  )
}
```

**Example: preload when feature flag is enabled**

```tsx
function FlagsProvider({ children, flags }: Props) {
  useEffect(() => {
    if (flags.editorEnabled && typeof window !== 'undefined') {
      void import('./monaco-editor').then(mod => mod.init())
    }
  }, [flags.editorEnabled])

  return <FlagsContext.Provider value={flags}>
    {children}
  </FlagsContext.Provider>
}
```

`typeof window !== 'undefined'` 체크는 SSR에서 preload 모듈이 번들에 포함되지 않게 막아 서버 번들 크기와 빌드 속도를 최적화한다.

---

## 3. Server-Side Performance

**Impact: HIGH**

서버 측 렌더링과 데이터 페칭을 최적화하면 서버 측 워터폴이 사라지고 응답 시간이 줄어든다.

### 3.1 Authenticate Server Actions Like API Routes

**Impact: CRITICAL (prevents unauthorized access to server mutations)**

Server Action(`"use server"` 함수)은 API 라우트와 마찬가지로 공개 엔드포인트로 노출된다. 미들웨어, 레이아웃 가드, 페이지 레벨 체크에만 의존하지 말고 각 Server Action **내부에서** 인증·인가를 항상 검증한다. Server Action은 직접 호출될 수 있다.

Next.js 문서는 명시적으로 다음과 같이 안내한다: "Treat Server Actions with the same security considerations as public-facing API endpoints, and verify if the user is allowed to perform a mutation."

**Incorrect: no authentication check**

```typescript
'use server'

export async function deleteUser(userId: string) {
  // Anyone can call this! No auth check
  await db.user.delete({ where: { id: userId } })
  return { success: true }
}
```

**Correct: authentication inside the action**

```typescript
'use server'

import { verifySession } from '@/lib/auth'
import { unauthorized } from '@/lib/errors'

export async function deleteUser(userId: string) {
  // Always check auth inside the action
  const session = await verifySession()
  
  if (!session) {
    throw unauthorized('Must be logged in')
  }
  
  // Check authorization too
  if (session.user.role !== 'admin' && session.user.id !== userId) {
    throw unauthorized('Cannot delete other users')
  }
  
  await db.user.delete({ where: { id: userId } })
  return { success: true }
}
```

**With input validation:**

```typescript
'use server'

import { verifySession } from '@/lib/auth'
import { z } from 'zod'

const updateProfileSchema = z.object({
  userId: z.string().uuid(),
  name: z.string().min(1).max(100),
  email: z.string().email()
})

export async function updateProfile(data: unknown) {
  // Validate input first
  const validated = updateProfileSchema.parse(data)
  
  // Then authenticate
  const session = await verifySession()
  if (!session) {
    throw new Error('Unauthorized')
  }
  
  // Then authorize
  if (session.user.id !== validated.userId) {
    throw new Error('Can only update own profile')
  }
  
  // Finally perform the mutation
  await db.user.update({
    where: { id: validated.userId },
    data: {
      name: validated.name,
      email: validated.email
    }
  })
  
  return { success: true }
}
```

Reference: [https://nextjs.org/docs/app/guides/authentication](https://nextjs.org/docs/app/guides/authentication)

### 3.2 Avoid Duplicate Serialization in RSC Props

**Impact: LOW (reduces network payload by avoiding duplicate serialization)**

RSC→client 직렬화는 객체 참조 기준으로 중복 제거된다. 같은 참조 = 한 번 직렬화, 새 참조 = 다시 직렬화. 변환(`.toSorted()`, `.filter()`, `.map()`)은 서버가 아니라 클라이언트에서 수행한다.

**Incorrect: duplicates array**

```tsx
// RSC: sends 6 strings (2 arrays × 3 items)
<ClientList usernames={usernames} usernamesOrdered={usernames.toSorted()} />
```

**Correct: sends 3 strings**

```tsx
// RSC: send once
<ClientList usernames={usernames} />

// Client: transform there
'use client'
const sorted = useMemo(() => [...usernames].sort(), [usernames])
```

**Nested deduplication behavior:**

```tsx
// string[] - duplicates everything
usernames={['a','b']} sorted={usernames.toSorted()} // sends 4 strings

// object[] - duplicates array structure only
users={[{id:1},{id:2}]} sorted={users.toSorted()} // sends 2 arrays + 2 unique objects (not 4)
```

중복 제거는 재귀적으로 동작한다. 데이터 타입에 따라 영향이 다르다.

- `string[]`, `number[]`, `boolean[]`: **HIGH impact** — 배열과 모든 원시값이 그대로 중복

- `object[]`: **LOW impact** — 배열은 중복되지만 중첩 객체는 참조로 중복 제거됨

**Operations breaking deduplication: create new references**

- Arrays: `.toSorted()`, `.filter()`, `.map()`, `.slice()`, `[...arr]`

- Objects: `{...obj}`, `Object.assign()`, `structuredClone()`, `JSON.parse(JSON.stringify())`

**More examples:**

```tsx
// ❌ Bad
<C users={users} active={users.filter(u => u.active)} />
<C product={product} productName={product.name} />

// ✅ Good
<C users={users} />
<C product={product} />
// Do filtering/destructuring in client
```

**Exception:** 변환이 비싸거나 클라이언트가 원본을 필요로 하지 않을 때는 파생 데이터를 전달한다.

### 3.3 Cross-Request LRU Caching

**Impact: HIGH (caches across requests)**

`React.cache()`는 단일 요청 내에서만 동작한다. 여러 순차 요청에서 공유되는 데이터(사용자가 버튼 A를 클릭한 후 버튼 B를 클릭)는 LRU 캐시를 사용한다.

**Implementation:**

```typescript
import { LRUCache } from 'lru-cache'

const cache = new LRUCache<string, any>({
  max: 1000,
  ttl: 5 * 60 * 1000  // 5 minutes
})

export async function getUser(id: string) {
  const cached = cache.get(id)
  if (cached) return cached

  const user = await db.user.findUnique({ where: { id } })
  cache.set(id, user)
  return user
}

// Request 1: DB query, result cached
// Request 2: cache hit, no DB query
```

순차 사용자 동작이 몇 초 안에 같은 데이터가 필요한 여러 엔드포인트를 호출할 때 사용한다.

**With Vercel's [Fluid Compute](https://vercel.com/docs/fluid-compute):** LRU 캐시는 동시 요청들이 같은 함수 인스턴스와 캐시를 공유할 수 있어 특히 효과적이다. 즉, Redis 같은 외부 저장소 없이도 요청 간에 캐시가 유지된다.

**In traditional serverless:** 각 호출이 격리되어 실행되므로 프로세스 간 캐시는 Redis를 고려한다.

Reference: [https://github.com/isaacs/node-lru-cache](https://github.com/isaacs/node-lru-cache)

### 3.4 Hoist Static I/O to Module Level

**Impact: HIGH (avoids repeated file/network I/O per request)**

라우트 핸들러나 서버 함수에서 정적 자산(폰트, 로고, 이미지, 설정 파일)을 로드할 때 I/O 연산을 모듈 레벨로 끌어올린다. 모듈 레벨 코드는 모듈이 처음 import될 때 한 번만 실행되고, 매 요청마다 실행되지 않는다. 이렇게 하면 매 호출마다 반복되는 파일 시스템 읽기나 네트워크 fetch가 사라진다.

**Incorrect: reads font file on every request**

```typescript
// app/api/og/route.tsx
import { ImageResponse } from 'next/og'

export async function GET(request: Request) {
  // Runs on EVERY request - expensive!
  const fontData = await fetch(
    new URL('./fonts/Inter.ttf', import.meta.url)
  ).then(res => res.arrayBuffer())

  const logoData = await fetch(
    new URL('./images/logo.png', import.meta.url)
  ).then(res => res.arrayBuffer())

  return new ImageResponse(
    <div style={{ fontFamily: 'Inter' }}>
      <img src={logoData} />
      Hello World
    </div>,
    { fonts: [{ name: 'Inter', data: fontData }] }
  )
}
```

**Correct: loads once at module initialization**

```typescript
// app/api/og/route.tsx
import { ImageResponse } from 'next/og'

// Module-level: runs ONCE when module is first imported
const fontData = fetch(
  new URL('./fonts/Inter.ttf', import.meta.url)
).then(res => res.arrayBuffer())

const logoData = fetch(
  new URL('./images/logo.png', import.meta.url)
).then(res => res.arrayBuffer())

export async function GET(request: Request) {
  // Await the already-started promises
  const [font, logo] = await Promise.all([fontData, logoData])

  return new ImageResponse(
    <div style={{ fontFamily: 'Inter' }}>
      <img src={logo} />
      Hello World
    </div>,
    { fonts: [{ name: 'Inter', data: font }] }
  )
}
```

**Correct: synchronous fs at module level**

```typescript
// app/api/og/route.tsx
import { ImageResponse } from 'next/og'
import { readFileSync } from 'fs'
import { join } from 'path'

// Synchronous read at module level - blocks only during module init
const fontData = readFileSync(
  join(process.cwd(), 'public/fonts/Inter.ttf')
)

const logoData = readFileSync(
  join(process.cwd(), 'public/images/logo.png')
)

export async function GET(request: Request) {
  return new ImageResponse(
    <div style={{ fontFamily: 'Inter' }}>
      <img src={logoData} />
      Hello World
    </div>,
    { fonts: [{ name: 'Inter', data: fontData }] }
  )
}
```

**Incorrect: reads config on every call**

```typescript
import fs from 'node:fs/promises'

export async function processRequest(data: Data) {
  const config = JSON.parse(
    await fs.readFile('./config.json', 'utf-8')
  )
  const template = await fs.readFile('./template.html', 'utf-8')

  return render(template, data, config)
}
```

**Correct: hoists config and template to module level**

```typescript
import fs from 'node:fs/promises'

const configPromise = fs
  .readFile('./config.json', 'utf-8')
  .then(JSON.parse)
const templatePromise = fs.readFile('./template.html', 'utf-8')

export async function processRequest(data: Data) {
  const [config, template] = await Promise.all([
    configPromise,
    templatePromise,
  ])

  return render(template, data, config)
}
```

이 패턴을 사용해야 할 때:

- OG 이미지 생성용 폰트 로드

- 정적 로고, 아이콘, 워터마크 로드

- 런타임에 변하지 않는 설정 파일 읽기

- 이메일 템플릿이나 기타 정적 템플릿 로드

- 모든 요청에서 동일한 정적 자산

이 패턴을 사용하면 안 될 때:

- 요청·사용자마다 달라지는 자산

- 런타임에 바뀔 수 있는 파일 (TTL 캐시를 대신 사용)

- 메모리에 상주시키기엔 너무 큰 파일

- 메모리에 남아선 안 되는 민감 데이터

Vercel [Fluid Compute](https://vercel.com/docs/fluid-compute) 환경에서는 동시 요청들이 같은 함수 인스턴스를 공유하므로 모듈 레벨 캐싱이 특히 효과적이다. 정적 자산은 cold start 패널티 없이 요청 간에 메모리에 머무른다.

전통적인 serverless에서는 각 cold start가 모듈 레벨 코드를 다시 실행하지만, 이후 warm 호출들은 인스턴스가 회수될 때까지 로드된 자산을 재사용한다.

### 3.5 Minimize Serialization at RSC Boundaries

**Impact: HIGH (reduces data transfer size)**

React Server/Client 경계는 모든 객체 속성을 문자열로 직렬화해서 HTML 응답과 이후 RSC 요청에 임베드한다. 이 직렬화 데이터는 페이지 무게와 로드 시간에 직접 영향을 미치므로 **크기가 매우 중요하다**. 클라이언트가 실제로 사용하는 필드만 전달한다.

**Incorrect: serializes all 50 fields**

```tsx
async function Page() {
  const user = await fetchUser()  // 50 fields
  return <Profile user={user} />
}

'use client'
function Profile({ user }: { user: User }) {
  return <div>{user.name}</div>  // uses 1 field
}
```

**Correct: serializes only 1 field**

```tsx
async function Page() {
  const user = await fetchUser()
  return <Profile name={user.name} />
}

'use client'
function Profile({ name }: { name: string }) {
  return <div>{name}</div>
}
```

### 3.6 Parallel Data Fetching with Component Composition

**Impact: CRITICAL (eliminates server-side waterfalls)**

React Server Component는 트리 안에서 순차적으로 실행된다. composition을 통해 구조를 재구성해 데이터 페칭을 병렬화한다.

**Incorrect: Sidebar waits for Page's fetch to complete**

```tsx
export default async function Page() {
  const header = await fetchHeader()
  return (
    <div>
      <div>{header}</div>
      <Sidebar />
    </div>
  )
}

async function Sidebar() {
  const items = await fetchSidebarItems()
  return <nav>{items.map(renderItem)}</nav>
}
```

**Correct: both fetch simultaneously**

```tsx
async function Header() {
  const data = await fetchHeader()
  return <div>{data}</div>
}

async function Sidebar() {
  const items = await fetchSidebarItems()
  return <nav>{items.map(renderItem)}</nav>
}

export default function Page() {
  return (
    <div>
      <Header />
      <Sidebar />
    </div>
  )
}
```

**Alternative with children prop:**

```tsx
async function Header() {
  const data = await fetchHeader()
  return <div>{data}</div>
}

async function Sidebar() {
  const items = await fetchSidebarItems()
  return <nav>{items.map(renderItem)}</nav>
}

function Layout({ children }: { children: ReactNode }) {
  return (
    <div>
      <Header />
      {children}
    </div>
  )
}

export default function Page() {
  return (
    <Layout>
      <Sidebar />
    </Layout>
  )
}
```

### 3.7 Parallel Nested Data Fetching

**Impact: CRITICAL (eliminates server-side waterfalls)**

중첩 데이터를 병렬로 페칭할 때는 의존하는 fetch를 각 항목의 promise 안에서 체이닝해, 느린 항목 하나가 나머지를 막지 않도록 한다.

**Incorrect: a single slow item blocks all nested fetches**

```tsx
const chats = await Promise.all(
  chatIds.map(id => getChat(id))
)

const chatAuthors = await Promise.all(
  chats.map(chat => getUser(chat.author))
)
```

100개 중 하나의 `getChat(id)`가 매우 느리면, 나머지 99개의 author는 데이터가 준비되어 있어도 로드를 시작할 수 없다.

**Correct: each item chains its own nested fetch**

```tsx
const chatAuthors = await Promise.all(
  chatIds.map(id => getChat(id).then(chat => getUser(chat.author)))
)
```

각 항목이 독립적으로 `getChat` → `getUser`를 체이닝하므로, 느린 chat이 다른 항목의 author 페칭을 막지 않는다.

### 3.8 Per-Request Deduplication with React.cache()

**Impact: MEDIUM (deduplicates within request)**

`React.cache()`는 서버 측 요청 중복 제거에 사용한다. 인증과 데이터베이스 쿼리에서 가장 큰 효과를 본다.

**Usage:**

```typescript
import { cache } from 'react'

export const getCurrentUser = cache(async () => {
  const session = await auth()
  if (!session?.user?.id) return null
  return await db.user.findUnique({
    where: { id: session.user.id }
  })
})
```

단일 요청 내에서 `getCurrentUser()`를 여러 번 호출해도 쿼리는 한 번만 실행된다.

**Avoid inline objects as arguments:**

`React.cache()`는 캐시 히트 판정에 얕은 동등성(`Object.is`)을 사용한다. 인라인 객체는 호출마다 새 참조를 만들어 캐시 히트를 막는다.

**Incorrect: always cache miss**

```typescript
const getUser = cache(async (params: { uid: number }) => {
  return await db.user.findUnique({ where: { id: params.uid } })
})

// Each call creates new object, never hits cache
getUser({ uid: 1 })
getUser({ uid: 1 })  // Cache miss, runs query again
```

**Correct: cache hit**

```typescript
const params = { uid: 1 }
getUser(params)  // Query runs
getUser(params)  // Cache hit (same reference)
```

객체를 전달해야 한다면 같은 참조를 전달한다.

**Next.js-Specific Note:**

Next.js에서는 `fetch` API가 자동으로 요청 메모이제이션으로 확장된다. 같은 URL과 옵션을 가진 요청은 단일 요청 내에서 자동으로 중복 제거되므로, `fetch` 호출에는 `React.cache()`가 필요하지 않다. 다만 다음 비동기 작업에는 여전히 `React.cache()`가 필요하다:

- 데이터베이스 쿼리 (Prisma, Drizzle 등)

- 무거운 연산

- 인증 체크

- 파일 시스템 연산

- 그 외 fetch가 아닌 모든 비동기 작업

컴포넌트 트리 전반에서 이러한 작업을 중복 제거하려면 `React.cache()`를 사용한다.

Reference: [https://react.dev/reference/react/cache](https://react.dev/reference/react/cache)

### 3.9 Use after() for Non-Blocking Operations

**Impact: MEDIUM (faster response times)**

응답이 전송된 이후에 실행할 작업은 Next.js의 `after()`로 스케줄링한다. 이렇게 하면 로깅, 분석, 기타 부수효과가 응답을 막지 않는다.

**Incorrect: blocks response**

```tsx
import { logUserAction } from '@/app/utils'

export async function POST(request: Request) {
  // Perform mutation
  await updateDatabase(request)
  
  // Logging blocks the response
  const userAgent = request.headers.get('user-agent') || 'unknown'
  await logUserAction({ userAgent })
  
  return new Response(JSON.stringify({ status: 'success' }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  })
}
```

**Correct: non-blocking**

```tsx
import { after } from 'next/server'
import { headers, cookies } from 'next/headers'
import { logUserAction } from '@/app/utils'

export async function POST(request: Request) {
  // Perform mutation
  await updateDatabase(request)
  
  // Log after response is sent
  after(async () => {
    const userAgent = (await headers()).get('user-agent') || 'unknown'
    const sessionCookie = (await cookies()).get('session-id')?.value || 'anonymous'
    
    logUserAction({ sessionCookie, userAgent })
  })
  
  return new Response(JSON.stringify({ status: 'success' }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  })
}
```

응답은 즉시 전송되고 로깅은 백그라운드에서 처리된다.

**Common use cases:**

- 분석 추적

- 감사 로그

- 알림 전송

- 캐시 무효화

- 정리 작업

**Important notes:**

- `after()`는 응답이 실패하거나 redirect되어도 실행된다.

- Server Actions, Route Handlers, Server Components에서 모두 동작한다.

Reference: [https://nextjs.org/docs/app/api-reference/functions/after](https://nextjs.org/docs/app/api-reference/functions/after)

---

## 4. Client-Side Data Fetching

**Impact: MEDIUM-HIGH**

자동 중복 제거와 효율적인 데이터 페칭 패턴은 중복 네트워크 요청을 줄인다.

### 4.1 Deduplicate Global Event Listeners

**Impact: LOW (single listener for N components)**

`useSWRSubscription()`을 사용해 컴포넌트 인스턴스 사이에서 전역 이벤트 리스너를 공유한다.

**Incorrect: N instances = N listeners**

```tsx
function useKeyboardShortcut(key: string, callback: () => void) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === key) {
        callback()
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [key, callback])
}
```

`useKeyboardShortcut` hook을 여러 번 사용하면 인스턴스마다 새로운 리스너가 등록된다.

**Correct: N instances = 1 listener**

```tsx
import useSWRSubscription from 'swr/subscription'

// Module-level Map to track callbacks per key
const keyCallbacks = new Map<string, Set<() => void>>()

function useKeyboardShortcut(key: string, callback: () => void) {
  // Register this callback in the Map
  useEffect(() => {
    if (!keyCallbacks.has(key)) {
      keyCallbacks.set(key, new Set())
    }
    keyCallbacks.get(key)!.add(callback)

    return () => {
      const set = keyCallbacks.get(key)
      if (set) {
        set.delete(callback)
        if (set.size === 0) {
          keyCallbacks.delete(key)
        }
      }
    }
  }, [key, callback])

  useSWRSubscription('global-keydown', () => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey && keyCallbacks.has(e.key)) {
        keyCallbacks.get(e.key)!.forEach(cb => cb())
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  })
}

function Profile() {
  // Multiple shortcuts will share the same listener
  useKeyboardShortcut('p', () => { /* ... */ }) 
  useKeyboardShortcut('k', () => { /* ... */ })
  // ...
}
```

### 4.2 Use Passive Event Listeners for Scrolling Performance

**Impact: MEDIUM (eliminates scroll delay caused by event listeners)**

touch와 wheel 이벤트 리스너에 `{ passive: true }`를 붙여 즉시 스크롤되도록 한다. 브라우저는 보통 리스너가 끝나기를 기다려 `preventDefault()` 호출 여부를 확인하는데, 이로 인해 스크롤 지연이 생긴다.

**Incorrect:**

```typescript
useEffect(() => {
  const handleTouch = (e: TouchEvent) => console.log(e.touches[0].clientX)
  const handleWheel = (e: WheelEvent) => console.log(e.deltaY)
  
  document.addEventListener('touchstart', handleTouch)
  document.addEventListener('wheel', handleWheel)
  
  return () => {
    document.removeEventListener('touchstart', handleTouch)
    document.removeEventListener('wheel', handleWheel)
  }
}, [])
```

**Correct:**

```typescript
useEffect(() => {
  const handleTouch = (e: TouchEvent) => console.log(e.touches[0].clientX)
  const handleWheel = (e: WheelEvent) => console.log(e.deltaY)
  
  document.addEventListener('touchstart', handleTouch, { passive: true })
  document.addEventListener('wheel', handleWheel, { passive: true })
  
  return () => {
    document.removeEventListener('touchstart', handleTouch)
    document.removeEventListener('wheel', handleWheel)
  }
}, [])
```

**Use passive when:** tracking/analytics, logging 등 `preventDefault()`를 호출하지 않는 모든 리스너.

**Don't use passive when:** custom swipe 제스처 구현, custom zoom 컨트롤, 또는 `preventDefault()`가 필요한 모든 리스너.

### 4.3 Use SWR for Automatic Deduplication

**Impact: MEDIUM-HIGH (automatic deduplication)**

SWR은 컴포넌트 인스턴스 간에 요청 중복 제거, 캐싱, 재검증을 가능하게 한다.

**Incorrect: no deduplication, each instance fetches**

```tsx
function UserList() {
  const [users, setUsers] = useState([])
  useEffect(() => {
    fetch('/api/users')
      .then(r => r.json())
      .then(setUsers)
  }, [])
}
```

**Correct: multiple instances share one request**

```tsx
import useSWR from 'swr'

function UserList() {
  const { data: users } = useSWR('/api/users', fetcher)
}
```

**For immutable data:**

```tsx
import { useImmutableSWR } from '@/lib/swr'

function StaticContent() {
  const { data } = useImmutableSWR('/api/config', fetcher)
}
```

**For mutations:**

```tsx
import { useSWRMutation } from 'swr/mutation'

function UpdateButton() {
  const { trigger } = useSWRMutation('/api/user', updateUser)
  return <button onClick={() => trigger()}>Update</button>
}
```

Reference: [https://swr.vercel.app](https://swr.vercel.app)

### 4.4 Version and Minimize localStorage Data

**Impact: MEDIUM (prevents schema conflicts, reduces storage size)**

키에 버전 prefix를 붙이고 필요한 필드만 저장한다. 스키마 충돌과 민감 정보의 우발적 저장을 방지한다.

**Incorrect:**

```typescript
// No version, stores everything, no error handling
localStorage.setItem('userConfig', JSON.stringify(fullUserObject))
const data = localStorage.getItem('userConfig')
```

**Correct:**

```typescript
const VERSION = 'v2'

function saveConfig(config: { theme: string; language: string }) {
  try {
    localStorage.setItem(`userConfig:${VERSION}`, JSON.stringify(config))
  } catch {
    // Throws in incognito/private browsing, quota exceeded, or disabled
  }
}

function loadConfig() {
  try {
    const data = localStorage.getItem(`userConfig:${VERSION}`)
    return data ? JSON.parse(data) : null
  } catch {
    return null
  }
}

// Migration from v1 to v2
function migrate() {
  try {
    const v1 = localStorage.getItem('userConfig:v1')
    if (v1) {
      const old = JSON.parse(v1)
      saveConfig({ theme: old.darkMode ? 'dark' : 'light', language: old.lang })
      localStorage.removeItem('userConfig:v1')
    }
  } catch {}
}
```

**Store minimal fields from server responses:**

```typescript
// User object has 20+ fields, only store what UI needs
function cachePrefs(user: FullUser) {
  try {
    localStorage.setItem('prefs:v1', JSON.stringify({
      theme: user.preferences.theme,
      notifications: user.preferences.notifications
    }))
  } catch {}
}
```

**Always wrap in try-catch:** `getItem()`, `setItem()`은 incognito/private 모드(Safari, Firefox), 할당량 초과, 비활성화 상태에서 throw를 던진다.

**Benefits:** versioning을 통한 스키마 진화, 저장 크기 감소, 토큰/PII/내부 플래그의 저장 방지.

---

## 5. Re-render Optimization

**Impact: MEDIUM**

불필요한 리렌더링을 줄이면 낭비되는 연산이 줄고 UI 반응성이 향상된다.

### 5.1 Calculate Derived State During Rendering

**Impact: MEDIUM (avoids redundant renders and state drift)**

현재 props/state로부터 계산할 수 있는 값이라면 state에 저장하거나 effect에서 갱신하지 않는다. 렌더링 중 파생해서 추가 렌더링과 state drift를 피한다. prop 변경에 반응하기 위한 목적만으로 effect에서 setState하지 말고, 파생 값이나 key 기반 reset을 사용한다.

**Incorrect: redundant state and effect**

```tsx
function Form() {
  const [firstName, setFirstName] = useState('First')
  const [lastName, setLastName] = useState('Last')
  const [fullName, setFullName] = useState('')

  useEffect(() => {
    setFullName(firstName + ' ' + lastName)
  }, [firstName, lastName])

  return <p>{fullName}</p>
}
```

**Correct: derive during render**

```tsx
function Form() {
  const [firstName, setFirstName] = useState('First')
  const [lastName, setLastName] = useState('Last')
  const fullName = firstName + ' ' + lastName

  return <p>{fullName}</p>
}
```

Reference: [https://react.dev/learn/you-might-not-need-an-effect](https://react.dev/learn/you-might-not-need-an-effect)

### 5.2 Defer State Reads to Usage Point

**Impact: MEDIUM (avoids unnecessary subscriptions)**

콜백 안에서만 읽는 동적 state(searchParams, localStorage)에는 구독하지 않는다.

**Incorrect: subscribes to all searchParams changes**

```tsx
function ShareButton({ chatId }: { chatId: string }) {
  const searchParams = useSearchParams()

  const handleShare = () => {
    const ref = searchParams.get('ref')
    shareChat(chatId, { ref })
  }

  return <button onClick={handleShare}>Share</button>
}
```

**Correct: reads on demand, no subscription**

```tsx
function ShareButton({ chatId }: { chatId: string }) {
  const handleShare = () => {
    const params = new URLSearchParams(window.location.search)
    const ref = params.get('ref')
    shareChat(chatId, { ref })
  }

  return <button onClick={handleShare}>Share</button>
}
```

### 5.3 Do not wrap a simple expression with a primitive result type in useMemo

**Impact: LOW-MEDIUM (wasted computation on every render)**

표현식이 단순(논리·산술 연산자가 적음)하고 결과 타입이 원시 타입(boolean, number, string)이라면 `useMemo`로 감싸지 않는다.

`useMemo` 호출과 의존성 비교 비용이 표현식 자체보다 더 클 수 있다.

**Incorrect:**

```tsx
function Header({ user, notifications }: Props) {
  const isLoading = useMemo(() => {
    return user.isLoading || notifications.isLoading
  }, [user.isLoading, notifications.isLoading])

  if (isLoading) return <Skeleton />
  // return some markup
}
```

**Correct:**

```tsx
function Header({ user, notifications }: Props) {
  const isLoading = user.isLoading || notifications.isLoading

  if (isLoading) return <Skeleton />
  // return some markup
}
```

### 5.4 Don't Define Components Inside Components

**Impact: HIGH (prevents remount on every render)**

다른 컴포넌트 안에서 컴포넌트를 정의하면 렌더링마다 새로운 컴포넌트 타입이 만들어진다. React는 매번 다른 컴포넌트로 인식해 완전히 remount하므로 모든 state와 DOM이 사라진다.

이렇게 하는 흔한 이유는 props를 전달하지 않고 부모 변수에 접근하기 위해서이다. 그 대신 항상 props를 전달한다.

**Incorrect: remounts on every render**

```tsx
function UserProfile({ user, theme }) {
  // Defined inside to access `theme` - BAD
  const Avatar = () => (
    <img
      src={user.avatarUrl}
      className={theme === 'dark' ? 'avatar-dark' : 'avatar-light'}
    />
  )

  // Defined inside to access `user` - BAD
  const Stats = () => (
    <div>
      <span>{user.followers} followers</span>
      <span>{user.posts} posts</span>
    </div>
  )

  return (
    <div>
      <Avatar />
      <Stats />
    </div>
  )
}
```

`UserProfile`이 렌더링될 때마다 `Avatar`와 `Stats`는 새로운 컴포넌트 타입이 된다. React는 이전 인스턴스를 unmount하고 새 인스턴스를 mount해, 내부 state가 사라지고 effect가 다시 실행되며 DOM 노드가 다시 생성된다.

**Correct: pass props instead**

```tsx
function Avatar({ src, theme }: { src: string; theme: string }) {
  return (
    <img
      src={src}
      className={theme === 'dark' ? 'avatar-dark' : 'avatar-light'}
    />
  )
}

function Stats({ followers, posts }: { followers: number; posts: number }) {
  return (
    <div>
      <span>{followers} followers</span>
      <span>{posts} posts</span>
    </div>
  )
}

function UserProfile({ user, theme }) {
  return (
    <div>
      <Avatar src={user.avatarUrl} theme={theme} />
      <Stats followers={user.followers} posts={user.posts} />
    </div>
  )
}
```

**Symptoms of this bug:**

- 키 입력마다 input 포커스가 사라진다

- 애니메이션이 예기치 않게 다시 시작된다

- 부모가 렌더링될 때마다 `useEffect` cleanup/setup이 실행된다

- 컴포넌트 내부 스크롤 위치가 리셋된다

### 5.5 Extract Default Non-primitive Parameter Value from Memoized Component to Constant

**Impact: MEDIUM (restores memoization by using a constant for default value)**

memoized 컴포넌트가 비원시 옵셔널 파라미터(배열, 함수, 객체 등)에 기본값을 가질 때, 그 파라미터 없이 컴포넌트를 호출하면 메모이제이션이 깨진다. 매 리렌더링마다 새 인스턴스가 만들어지고 `memo()`의 strict equality 비교를 통과하지 못하기 때문이다.

기본값을 상수로 추출해서 해결한다.

**Incorrect: `onClick` has different values on every rerender**

```tsx
const UserAvatar = memo(function UserAvatar({ onClick = () => {} }: { onClick?: () => void }) {
  // ...
})

// Used without optional onClick
<UserAvatar />
```

**Correct: stable default value**

```tsx
const NOOP = () => {};

const UserAvatar = memo(function UserAvatar({ onClick = NOOP }: { onClick?: () => void }) {
  // ...
})

// Used without optional onClick
<UserAvatar />
```

### 5.6 Extract to Memoized Components

**Impact: MEDIUM (enables early returns)**

비싼 작업을 memoized 컴포넌트로 추출해, 연산 전에 early return이 가능하도록 한다.

**Incorrect: computes avatar even when loading**

```tsx
function Profile({ user, loading }: Props) {
  const avatar = useMemo(() => {
    const id = computeAvatarId(user)
    return <Avatar id={id} />
  }, [user])

  if (loading) return <Skeleton />
  return <div>{avatar}</div>
}
```

**Correct: skips computation when loading**

```tsx
const UserAvatar = memo(function UserAvatar({ user }: { user: User }) {
  const id = useMemo(() => computeAvatarId(user), [user])
  return <Avatar id={id} />
})

function Profile({ user, loading }: Props) {
  if (loading) return <Skeleton />
  return (
    <div>
      <UserAvatar user={user} />
    </div>
  )
}
```

**Note:** 프로젝트가 [React Compiler](https://react.dev/learn/react-compiler)를 사용한다면 `memo()`와 `useMemo()`로 수동 메모이제이션을 할 필요가 없다. 컴파일러가 리렌더링을 자동으로 최적화한다.

### 5.7 Narrow Effect Dependencies

**Impact: LOW (minimizes effect re-runs)**

effect 재실행을 줄이려면 객체 대신 원시 의존성을 명시한다.

**Incorrect: re-runs on any user field change**

```tsx
useEffect(() => {
  console.log(user.id)
}, [user])
```

**Correct: re-runs only when id changes**

```tsx
useEffect(() => {
  console.log(user.id)
}, [user.id])
```

**For derived state, compute outside effect:**

```tsx
// Incorrect: runs on width=767, 766, 765...
useEffect(() => {
  if (width < 768) {
    enableMobileMode()
  }
}, [width])

// Correct: runs only on boolean transition
const isMobile = width < 768
useEffect(() => {
  if (isMobile) {
    enableMobileMode()
  }
}, [isMobile])
```

### 5.8 Put Interaction Logic in Event Handlers

**Impact: MEDIUM (avoids effect re-runs and duplicate side effects)**

특정 사용자 동작(submit, click, drag)으로 트리거되는 부수효과는 그 이벤트 핸들러 안에서 실행한다. 동작을 state + effect로 모델링하지 않는다 — 무관한 변경에도 effect가 다시 실행되고 동작이 중복될 수 있다.

**Incorrect: event modeled as state + effect**

```tsx
function Form() {
  const [submitted, setSubmitted] = useState(false)
  const theme = useContext(ThemeContext)

  useEffect(() => {
    if (submitted) {
      post('/api/register')
      showToast('Registered', theme)
    }
  }, [submitted, theme])

  return <button onClick={() => setSubmitted(true)}>Submit</button>
}
```

**Correct: do it in the handler**

```tsx
function Form() {
  const theme = useContext(ThemeContext)

  function handleSubmit() {
    post('/api/register')
    showToast('Registered', theme)
  }

  return <button onClick={handleSubmit}>Submit</button>
}
```

Reference: [https://react.dev/learn/removing-effect-dependencies#should-this-code-move-to-an-event-handler](https://react.dev/learn/removing-effect-dependencies#should-this-code-move-to-an-event-handler)

### 5.9 Split Combined Hook Computations

**Impact: MEDIUM (avoids recomputing independent steps)**

서로 다른 의존성을 가진 독립 작업이 한 hook에 들어 있다면 별도 hook으로 분리한다. 합쳐진 hook은 어떤 의존성이 변경되어도 모든 작업을 다시 실행하므로, 변경된 값을 사용하지 않는 작업까지 재계산된다.

**Incorrect: changing `sortOrder` recomputes filtering**

```tsx
const sortedProducts = useMemo(() => {
  const filtered = products.filter((p) => p.category === category)
  const sorted = filtered.toSorted((a, b) =>
    sortOrder === "asc" ? a.price - b.price : b.price - a.price
  )
  return sorted
}, [products, category, sortOrder])
```

**Correct: filtering only recomputes when products or category change**

```tsx
const filteredProducts = useMemo(
  () => products.filter((p) => p.category === category),
  [products, category]
)

const sortedProducts = useMemo(
  () =>
    filteredProducts.toSorted((a, b) =>
      sortOrder === "asc" ? a.price - b.price : b.price - a.price
    ),
  [filteredProducts, sortOrder]
)
```

서로 무관한 부수효과를 합친 `useEffect`에도 같은 패턴을 적용한다.

**Incorrect: both effects run when either dependency changes**

```tsx
useEffect(() => {
  analytics.trackPageView(pathname)
  document.title = `${pageTitle} | My App`
}, [pathname, pageTitle])
```

**Correct: effects run independently**

```tsx
useEffect(() => {
  analytics.trackPageView(pathname)
}, [pathname])

useEffect(() => {
  document.title = `${pageTitle} | My App`
}, [pageTitle])
```

**Note:** 프로젝트가 [React Compiler](https://react.dev/learn/react-compiler)를 사용한다면 의존성 추적을 자동 최적화하므로 일부 케이스는 자동으로 처리된다.

### 5.10 Subscribe to Derived State

**Impact: MEDIUM (reduces re-render frequency)**

연속 값 대신 파생된 boolean 상태에 구독해 리렌더링 빈도를 줄인다.

**Incorrect: re-renders on every pixel change**

```tsx
function Sidebar() {
  const width = useWindowWidth()  // updates continuously
  const isMobile = width < 768
  return <nav className={isMobile ? 'mobile' : 'desktop'} />
}
```

**Correct: re-renders only when boolean changes**

```tsx
function Sidebar() {
  const isMobile = useMediaQuery('(max-width: 767px)')
  return <nav className={isMobile ? 'mobile' : 'desktop'} />
}
```

### 5.11 Use Functional setState Updates

**Impact: MEDIUM (prevents stale closures and unnecessary callback recreations)**

현재 state 값을 기반으로 state를 업데이트할 때 state 변수를 직접 참조하지 말고 setState의 functional update 형태를 사용한다. stale closure를 방지하고 불필요한 의존성을 제거하며 안정적인 콜백 참조를 만든다.

**Incorrect: requires state as dependency**

```tsx
function TodoList() {
  const [items, setItems] = useState(initialItems)
  
  // Callback must depend on items, recreated on every items change
  const addItems = useCallback((newItems: Item[]) => {
    setItems([...items, ...newItems])
  }, [items])  // ❌ items dependency causes recreations
  
  // Risk of stale closure if dependency is forgotten
  const removeItem = useCallback((id: string) => {
    setItems(items.filter(item => item.id !== id))
  }, [])  // ❌ Missing items dependency - will use stale items!
  
  return <ItemsEditor items={items} onAdd={addItems} onRemove={removeItem} />
}
```

첫 번째 콜백은 `items`가 변경될 때마다 다시 만들어져 자식 컴포넌트가 불필요하게 리렌더링된다. 두 번째 콜백은 stale closure 버그가 있다 — 항상 초기 `items` 값을 참조한다.

**Correct: stable callbacks, no stale closures**

```tsx
function TodoList() {
  const [items, setItems] = useState(initialItems)
  
  // Stable callback, never recreated
  const addItems = useCallback((newItems: Item[]) => {
    setItems(curr => [...curr, ...newItems])
  }, [])  // ✅ No dependencies needed
  
  // Always uses latest state, no stale closure risk
  const removeItem = useCallback((id: string) => {
    setItems(curr => curr.filter(item => item.id !== id))
  }, [])  // ✅ Safe and stable
  
  return <ItemsEditor items={items} onAdd={addItems} onRemove={removeItem} />
}
```

**Benefits:**

1. **안정적인 콜백 참조** — state 변경 시 콜백을 다시 만들 필요가 없다

2. **stale closure 없음** — 항상 최신 state 값에 대해 동작한다

3. **의존성 감소** — 의존성 배열이 단순해지고 메모리 누수가 줄어든다

4. **버그 예방** — React closure 버그의 가장 흔한 원인을 제거한다

**When to use functional updates:**

- 현재 state 값에 의존하는 모든 setState

- state가 필요한 useCallback/useMemo 내부

- state를 참조하는 이벤트 핸들러

- state를 갱신하는 비동기 연산

**When direct updates are fine:**

- 정적 값으로 state 설정: `setCount(0)`

- props/arguments로만 state 설정: `setName(newName)`

- state가 이전 값에 의존하지 않음

**Note:** 프로젝트가 [React Compiler](https://react.dev/learn/react-compiler)를 사용한다면 일부는 자동 최적화되지만, 정확성과 stale closure 버그 방지를 위해 functional update가 여전히 권장된다.

### 5.12 Use Lazy State Initialization

**Impact: MEDIUM (wasted computation on every render)**

비싼 초기값에는 함수를 `useState`에 전달한다. 함수 형태가 아니면 값이 한 번만 사용되더라도 매 렌더링마다 initializer가 실행된다.

**Incorrect: runs on every render**

```tsx
function FilteredList({ items }: { items: Item[] }) {
  // buildSearchIndex() runs on EVERY render, even after initialization
  const [searchIndex, setSearchIndex] = useState(buildSearchIndex(items))
  const [query, setQuery] = useState('')
  
  // When query changes, buildSearchIndex runs again unnecessarily
  return <SearchResults index={searchIndex} query={query} />
}

function UserProfile() {
  // JSON.parse runs on every render
  const [settings, setSettings] = useState(
    JSON.parse(localStorage.getItem('settings') || '{}')
  )
  
  return <SettingsForm settings={settings} onChange={setSettings} />
}
```

**Correct: runs only once**

```tsx
function FilteredList({ items }: { items: Item[] }) {
  // buildSearchIndex() runs ONLY on initial render
  const [searchIndex, setSearchIndex] = useState(() => buildSearchIndex(items))
  const [query, setQuery] = useState('')
  
  return <SearchResults index={searchIndex} query={query} />
}

function UserProfile() {
  // JSON.parse runs only on initial render
  const [settings, setSettings] = useState(() => {
    const stored = localStorage.getItem('settings')
    return stored ? JSON.parse(stored) : {}
  })
  
  return <SettingsForm settings={settings} onChange={setSettings} />
}
```

localStorage/sessionStorage에서 초기값을 읽거나, 데이터 구조(인덱스, 맵)를 빌드하거나, DOM에서 읽거나, 무거운 변환을 수행할 때 lazy 초기화를 사용한다.

단순 원시값(`useState(0)`), 직접 참조(`useState(props.value)`), 저비용 리터럴(`useState({})`)에는 함수 형태가 불필요하다.

### 5.13 Use Transitions for Non-Urgent Updates

**Impact: MEDIUM (maintains UI responsiveness)**

자주 발생하는 비긴급 state 업데이트를 transition으로 표시해 UI 반응성을 유지한다.

**Incorrect: blocks UI on every scroll**

```tsx
function ScrollTracker() {
  const [scrollY, setScrollY] = useState(0)
  useEffect(() => {
    const handler = () => setScrollY(window.scrollY)
    window.addEventListener('scroll', handler, { passive: true })
    return () => window.removeEventListener('scroll', handler)
  }, [])
}
```

**Correct: non-blocking updates**

```tsx
import { startTransition } from 'react'

function ScrollTracker() {
  const [scrollY, setScrollY] = useState(0)
  useEffect(() => {
    const handler = () => {
      startTransition(() => setScrollY(window.scrollY))
    }
    window.addEventListener('scroll', handler, { passive: true })
    return () => window.removeEventListener('scroll', handler)
  }, [])
}
```

### 5.14 Use useDeferredValue for Expensive Derived Renders

**Impact: MEDIUM (keeps input responsive during heavy computation)**

사용자 입력이 비싼 연산이나 렌더링을 트리거할 때 `useDeferredValue`로 입력 반응성을 유지한다. deferred 값은 살짝 지연되어, React가 입력 업데이트를 우선 처리하고 idle 상태일 때 비싼 결과를 렌더링한다.

**Incorrect: input feels laggy while filtering**

```tsx
function Search({ items }: { items: Item[] }) {
  const [query, setQuery] = useState('')
  const filtered = items.filter(item => fuzzyMatch(item, query))

  return (
    <>
      <input value={query} onChange={e => setQuery(e.target.value)} />
      <ResultsList results={filtered} />
    </>
  )
}
```

**Correct: input stays snappy, results render when ready**

```tsx
function Search({ items }: { items: Item[] }) {
  const [query, setQuery] = useState('')
  const deferredQuery = useDeferredValue(query)
  const filtered = useMemo(
    () => items.filter(item => fuzzyMatch(item, deferredQuery)),
    [items, deferredQuery]
  )
  const isStale = query !== deferredQuery

  return (
    <>
      <input value={query} onChange={e => setQuery(e.target.value)} />
      <div style={{ opacity: isStale ? 0.7 : 1 }}>
        <ResultsList results={filtered} />
      </div>
    </>
  )
}
```

**When to use:**

- 큰 리스트의 필터링·검색

- 입력에 반응하는 비싼 시각화(차트, 그래프)

- 눈에 띄는 렌더 지연을 일으키는 모든 파생 state

**Note:** 비싼 연산은 deferred 값을 의존성으로 하는 `useMemo`로 감싸야 한다. 그렇지 않으면 매 렌더링마다 실행된다.

Reference: [https://react.dev/reference/react/useDeferredValue](https://react.dev/reference/react/useDeferredValue)

### 5.15 Use useRef for Transient Values

**Impact: MEDIUM (avoids unnecessary re-renders on frequent updates)**

값이 자주 바뀌고 매 업데이트마다 리렌더링하고 싶지 않다면(예: 마우스 추적, interval, 임시 플래그) `useState` 대신 `useRef`에 저장한다. 컴포넌트 state는 UI를 위한 것이고, ref는 임시적인 DOM 인접 값에 사용한다. ref 업데이트는 리렌더링을 트리거하지 않는다.

**Incorrect: renders every update**

```tsx
function Tracker() {
  const [lastX, setLastX] = useState(0)

  useEffect(() => {
    const onMove = (e: MouseEvent) => setLastX(e.clientX)
    window.addEventListener('mousemove', onMove)
    return () => window.removeEventListener('mousemove', onMove)
  }, [])

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: lastX,
        width: 8,
        height: 8,
        background: 'black',
      }}
    />
  )
}
```

**Correct: no re-render for tracking**

```tsx
function Tracker() {
  const lastXRef = useRef(0)
  const dotRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      lastXRef.current = e.clientX
      const node = dotRef.current
      if (node) {
        node.style.transform = `translateX(${e.clientX}px)`
      }
    }
    window.addEventListener('mousemove', onMove)
    return () => window.removeEventListener('mousemove', onMove)
  }, [])

  return (
    <div
      ref={dotRef}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: 8,
        height: 8,
        background: 'black',
        transform: 'translateX(0px)',
      }}
    />
  )
}
```

---

## 6. Rendering Performance

**Impact: MEDIUM**

렌더링 과정을 최적화하면 브라우저가 처리해야 할 작업량이 줄어든다.

### 6.1 Animate SVG Wrapper Instead of SVG Element

**Impact: LOW (enables hardware acceleration)**

다수의 브라우저는 SVG 요소의 CSS3 애니메이션에 대해 하드웨어 가속을 지원하지 않는다. SVG를 `<div>`로 감싸고 wrapper에 애니메이션을 건다.

**Incorrect: animating SVG directly - no hardware acceleration**

```tsx
function LoadingSpinner() {
  return (
    <svg 
      className="animate-spin"
      width="24" 
      height="24" 
      viewBox="0 0 24 24"
    >
      <circle cx="12" cy="12" r="10" stroke="currentColor" />
    </svg>
  )
}
```

**Correct: animating wrapper div - hardware accelerated**

```tsx
function LoadingSpinner() {
  return (
    <div className="animate-spin">
      <svg 
        width="24" 
        height="24" 
        viewBox="0 0 24 24"
      >
        <circle cx="12" cy="12" r="10" stroke="currentColor" />
      </svg>
    </div>
  )
}
```

모든 CSS transform과 transition(`transform`, `opacity`, `translate`, `scale`, `rotate`)에 적용된다. wrapper div를 사용하면 브라우저가 GPU 가속으로 더 부드러운 애니메이션을 만든다.

### 6.2 CSS content-visibility for Long Lists

**Impact: HIGH (faster initial render)**

`content-visibility: auto`를 적용해 화면 밖 영역의 렌더링을 지연시킨다.

**CSS:**

```css
.message-item {
  content-visibility: auto;
  contain-intrinsic-size: 0 80px;
}
```

**Example:**

```tsx
function MessageList({ messages }: { messages: Message[] }) {
  return (
    <div className="overflow-y-auto h-screen">
      {messages.map(msg => (
        <div key={msg.id} className="message-item">
          <Avatar user={msg.author} />
          <div>{msg.content}</div>
        </div>
      ))}
    </div>
  )
}
```

1000개의 메시지에서 브라우저는 화면 밖 약 990개에 대해 layout/paint를 건너뛴다 (초기 렌더링이 약 10배 빨라진다).

### 6.3 Hoist Static JSX Elements

**Impact: LOW (avoids re-creation)**

정적 JSX는 컴포넌트 밖으로 빼서 재생성을 피한다.

**Incorrect: recreates element every render**

```tsx
function LoadingSkeleton() {
  return <div className="animate-pulse h-20 bg-gray-200" />
}

function Container() {
  return (
    <div>
      {loading && <LoadingSkeleton />}
    </div>
  )
}
```

**Correct: reuses same element**

```tsx
const loadingSkeleton = (
  <div className="animate-pulse h-20 bg-gray-200" />
)

function Container() {
  return (
    <div>
      {loading && loadingSkeleton}
    </div>
  )
}
```

크고 정적인 SVG 노드처럼 재생성 비용이 큰 요소에 특히 도움이 된다.

**Note:** 프로젝트가 [React Compiler](https://react.dev/learn/react-compiler)를 사용한다면 정적 JSX 요소를 자동으로 호이스팅하고 컴포넌트 리렌더링을 최적화하므로 수동 호이스팅이 필요 없다.

### 6.4 Optimize SVG Precision

**Impact: LOW (reduces file size)**

SVG 좌표 정밀도를 줄여 파일 크기를 감소시킨다. 최적 정밀도는 viewBox 크기에 따라 다르지만, 일반적으로 정밀도 축소를 고려한다.

**Incorrect: excessive precision**

```svg
<path d="M 10.293847 20.847362 L 30.938472 40.192837" />
```

**Correct: 1 decimal place**

```svg
<path d="M 10.3 20.8 L 30.9 40.2" />
```

**Automate with SVGO:**

```bash
npx svgo --precision=1 --multipass icon.svg
```

### 6.5 Prevent Hydration Mismatch Without Flickering

**Impact: MEDIUM (avoids visual flicker and hydration errors)**

클라이언트 측 저장소(localStorage, cookie)에 의존하는 콘텐츠를 렌더링할 때, React가 hydrate되기 전에 DOM을 갱신하는 동기 스크립트를 주입해 SSR 깨짐과 hydration 후 깜빡임을 모두 피한다.

**Incorrect: breaks SSR**

```tsx
function ThemeWrapper({ children }: { children: ReactNode }) {
  // localStorage is not available on server - throws error
  const theme = localStorage.getItem('theme') || 'light'
  
  return (
    <div className={theme}>
      {children}
    </div>
  )
}
```

서버에서는 `localStorage`가 정의되지 않아 SSR이 실패한다.

**Incorrect: visual flickering**

```tsx
function ThemeWrapper({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState('light')
  
  useEffect(() => {
    // Runs after hydration - causes visible flash
    const stored = localStorage.getItem('theme')
    if (stored) {
      setTheme(stored)
    }
  }, [])
  
  return (
    <div className={theme}>
      {children}
    </div>
  )
}
```

컴포넌트가 먼저 기본값(`light`)으로 렌더링된 후 hydration 직후 갱신되어, 잘못된 콘텐츠가 잠깐 보이는 깜빡임이 생긴다.

**Correct: no flicker, no hydration mismatch**

```tsx
function ThemeWrapper({ children }: { children: ReactNode }) {
  return (
    <>
      <div id="theme-wrapper">
        {children}
      </div>
      <script
        dangerouslySetInnerHTML={{
          __html: `
            (function() {
              try {
                var theme = localStorage.getItem('theme') || 'light';
                var el = document.getElementById('theme-wrapper');
                if (el) el.className = theme;
              } catch (e) {}
            })();
          `,
        }}
      />
    </>
  )
}
```

inline 스크립트가 요소가 표시되기 전에 동기적으로 실행되어 DOM이 이미 올바른 값을 가지도록 보장한다. 깜빡임도 없고 hydration mismatch도 없다.

이 패턴은 테마 토글, 사용자 환경 설정, 인증 상태, 그리고 기본값 깜빡임 없이 즉시 렌더링되어야 하는 모든 클라이언트 전용 데이터에 특히 유용하다.

### 6.6 Suppress Expected Hydration Mismatches

**Impact: LOW-MEDIUM (avoids noisy hydration warnings for known differences)**

SSR 프레임워크(예: Next.js)에서는 일부 값이 서버와 클라이언트에서 의도적으로 다르다(랜덤 ID, 날짜, locale/timezone 포맷). 이러한 *예상된* 불일치에 대해서는 동적 텍스트를 `suppressHydrationWarning`이 적용된 요소로 감싸 시끄러운 경고를 막는다. 진짜 버그를 숨기는 데 사용하지 말고, 남용하지 않는다.

**Incorrect: known mismatch warnings**

```tsx
function Timestamp() {
  return <span>{new Date().toLocaleString()}</span>
}
```

**Correct: suppress expected mismatch only**

```tsx
function Timestamp() {
  return (
    <span suppressHydrationWarning>
      {new Date().toLocaleString()}
    </span>
  )
}
```

### 6.7 Use Activity Component for Show/Hide

**Impact: MEDIUM (preserves state/DOM)**

자주 보임/숨김이 토글되는 비싼 컴포넌트는 React의 `<Activity>`로 감싸 state/DOM을 보존한다.

**Usage:**

```tsx
import { Activity } from 'react'

function Dropdown({ isOpen }: Props) {
  return (
    <Activity mode={isOpen ? 'visible' : 'hidden'}>
      <ExpensiveMenu />
    </Activity>
  )
}
```

비싼 리렌더링과 state 손실을 피한다.

### 6.8 Use defer or async on Script Tags

**Impact: HIGH (eliminates render-blocking)**

`defer`나 `async` 없이 사용하는 script 태그는 스크립트가 다운로드·실행되는 동안 HTML 파싱을 막는다. First Contentful Paint와 Time to Interactive가 지연된다.

- **`defer`**: 병렬 다운로드, HTML 파싱 완료 후 실행, 실행 순서 보장

- **`async`**: 병렬 다운로드, 준비되는 즉시 실행, 순서 보장 없음

DOM이나 다른 스크립트에 의존하는 스크립트는 `defer`, analytics 같은 독립 스크립트는 `async`를 사용한다.

**Incorrect: blocks rendering**

```tsx
export default function Document() {
  return (
    <html>
      <head>
        <script src="https://example.com/analytics.js" />
        <script src="/scripts/utils.js" />
      </head>
      <body>{/* content */}</body>
    </html>
  )
}
```

**Correct: non-blocking**

```tsx
import Script from 'next/script'

export default function Page() {
  return (
    <>
      <Script src="https://example.com/analytics.js" strategy="afterInteractive" />
      <Script src="/scripts/utils.js" strategy="beforeInteractive" />
    </>
  )
}
```

**Note:** Next.js에서는 raw script 태그 대신 `next/script` 컴포넌트와 `strategy` prop을 사용한다.

Reference: [https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script#defer](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script#defer)

### 6.9 Use Explicit Conditional Rendering

**Impact: LOW (prevents rendering 0 or NaN)**

조건이 `0`, `NaN`, 기타 falsy이지만 렌더링되는 값일 수 있다면 `&&` 대신 명시적 ternary(`? :`)를 사용한다.

**Incorrect: renders "0" when count is 0**

```tsx
function Badge({ count }: { count: number }) {
  return (
    <div>
      {count && <span className="badge">{count}</span>}
    </div>
  )
}

// When count = 0, renders: <div>0</div>
// When count = 5, renders: <div><span class="badge">5</span></div>
```

**Correct: renders nothing when count is 0**

```tsx
function Badge({ count }: { count: number }) {
  return (
    <div>
      {count > 0 ? <span className="badge">{count}</span> : null}
    </div>
  )
}

// When count = 0, renders: <div></div>
// When count = 5, renders: <div><span class="badge">5</span></div>
```

### 6.10 Use React DOM Resource Hints

**Impact: HIGH (reduces load time for critical resources)**

React DOM은 브라우저에 필요할 리소스를 힌트로 알리는 API를 제공한다. 서버 컴포넌트에서 사용하면 클라이언트가 HTML을 받기도 전에 리소스 로드를 시작할 수 있어 특히 유용하다.

- **`prefetchDNS(href)`**: 연결할 도메인의 DNS 해석

- **`preconnect(href)`**: 서버에 연결 수립 (DNS + TCP + TLS)

- **`preload(href, options)`**: 곧 사용할 리소스(stylesheet, font, script, image) fetch

- **`preloadModule(href)`**: 곧 사용할 ES module fetch

- **`preinit(href, options)`**: stylesheet나 script를 fetch하고 평가

- **`preinitModule(href)`**: ES module을 fetch하고 평가

**Example: preconnect to third-party APIs**

```tsx
import { preconnect, prefetchDNS } from 'react-dom'

export default function App() {
  prefetchDNS('https://analytics.example.com')
  preconnect('https://api.example.com')

  return <main>{/* content */}</main>
}
```

**Example: preload critical fonts and styles**

```tsx
import { preload, preinit } from 'react-dom'

export default function RootLayout({ children }) {
  // Preload font file
  preload('/fonts/inter.woff2', { as: 'font', type: 'font/woff2', crossOrigin: 'anonymous' })

  // Fetch and apply critical stylesheet immediately
  preinit('/styles/critical.css', { as: 'style' })

  return (
    <html>
      <body>{children}</body>
    </html>
  )
}
```

**Example: preload modules for code-split routes**

```tsx
import { preloadModule, preinitModule } from 'react-dom'

function Navigation() {
  const preloadDashboard = () => {
    preloadModule('/dashboard.js', { as: 'script' })
  }

  return (
    <nav>
      <a href="/dashboard" onMouseEnter={preloadDashboard}>
        Dashboard
      </a>
    </nav>
  )
}
```

**When to use each:**

| API | Use case |

|-----|----------|

| `prefetchDNS` | 나중에 연결할 third-party 도메인 |

| `preconnect` | 즉시 fetch할 API나 CDN |

| `preload` | 현재 페이지에 필요한 핵심 리소스 |

| `preloadModule` | 다음 네비게이션에 필요할 가능성이 높은 JS 모듈 |

| `preinit` | 일찍 실행되어야 하는 stylesheet/script |

| `preinitModule` | 일찍 실행되어야 하는 ES module |

Reference: [https://react.dev/reference/react-dom#resource-preloading-apis](https://react.dev/reference/react-dom#resource-preloading-apis)

### 6.11 Use useTransition Over Manual Loading States

**Impact: LOW (reduces re-renders and improves code clarity)**

수동 `useState`로 로딩 상태를 관리하지 말고 `useTransition`을 사용한다. 내장 `isPending` 상태를 제공하고 transition을 자동으로 관리한다.

**Incorrect: manual loading state**

```tsx
function SearchResults() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState([])
  const [isLoading, setIsLoading] = useState(false)

  const handleSearch = async (value: string) => {
    setIsLoading(true)
    setQuery(value)
    const data = await fetchResults(value)
    setResults(data)
    setIsLoading(false)
  }

  return (
    <>
      <input onChange={(e) => handleSearch(e.target.value)} />
      {isLoading && <Spinner />}
      <ResultsList results={results} />
    </>
  )
}
```

**Correct: useTransition with built-in pending state**

```tsx
import { useTransition, useState } from 'react'

function SearchResults() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState([])
  const [isPending, startTransition] = useTransition()

  const handleSearch = (value: string) => {
    setQuery(value) // Update input immediately
    
    startTransition(async () => {
      // Fetch and update results
      const data = await fetchResults(value)
      setResults(data)
    })
  }

  return (
    <>
      <input onChange={(e) => handleSearch(e.target.value)} />
      {isPending && <Spinner />}
      <ResultsList results={results} />
    </>
  )
}
```

**Benefits:**

- **자동 pending state**: `setIsLoading(true/false)`을 수동으로 관리할 필요가 없다

- **에러 복원력**: transition이 throw해도 pending 상태가 정확히 reset된다

- **반응성 향상**: 업데이트 중에도 UI 반응성이 유지된다

- **interrupt 처리**: 새 transition이 pending 중인 것을 자동 취소한다

Reference: [https://react.dev/reference/react/useTransition](https://react.dev/reference/react/useTransition)

---

## 7. JavaScript Performance

**Impact: LOW-MEDIUM**

핫 패스에 대한 마이크로 최적화가 누적되면 의미 있는 개선이 된다.

### 7.1 Avoid Layout Thrashing

**Impact: MEDIUM (prevents forced synchronous layouts and reduces performance bottlenecks)**

스타일 쓰기와 레이아웃 읽기를 교차로 실행하지 않는다. 스타일 변경 사이에 레이아웃 속성(`offsetWidth`, `getBoundingClientRect()`, `getComputedStyle()` 등)을 읽으면 브라우저가 동기 reflow를 강제 트리거한다.

**This is OK: browser batches style changes**

```typescript
function updateElementStyles(element: HTMLElement) {
  // Each line invalidates style, but browser batches the recalculation
  element.style.width = '100px'
  element.style.height = '200px'
  element.style.backgroundColor = 'blue'
  element.style.border = '1px solid black'
}
```

**Incorrect: interleaved reads and writes force reflows**

```typescript
function layoutThrashing(element: HTMLElement) {
  element.style.width = '100px'
  const width = element.offsetWidth  // Forces reflow
  element.style.height = '200px'
  const height = element.offsetHeight  // Forces another reflow
}
```

**Correct: batch writes, then read once**

```typescript
function updateElementStyles(element: HTMLElement) {
  // Batch all writes together
  element.style.width = '100px'
  element.style.height = '200px'
  element.style.backgroundColor = 'blue'
  element.style.border = '1px solid black'
  
  // Read after all writes are done (single reflow)
  const { width, height } = element.getBoundingClientRect()
}
```

**Correct: batch reads, then writes**

```typescript
function updateElementStyles(element: HTMLElement) {
  element.classList.add('highlighted-box')
  
  const { width, height } = element.getBoundingClientRect()
}
```

**Better: use CSS classes**

**React example:**

```tsx
// Incorrect: interleaving style changes with layout queries
function Box({ isHighlighted }: { isHighlighted: boolean }) {
  const ref = useRef<HTMLDivElement>(null)
  
  useEffect(() => {
    if (ref.current && isHighlighted) {
      ref.current.style.width = '100px'
      const width = ref.current.offsetWidth // Forces layout
      ref.current.style.height = '200px'
    }
  }, [isHighlighted])
  
  return <div ref={ref}>Content</div>
}

// Correct: toggle class
function Box({ isHighlighted }: { isHighlighted: boolean }) {
  return (
    <div className={isHighlighted ? 'highlighted-box' : ''}>
      Content
    </div>
  )
}
```

가능하면 inline 스타일보다 CSS 클래스를 사용한다. CSS 파일은 브라우저에 캐시되고, 클래스는 관심사 분리가 더 명확하며 유지보수에 유리하다.

레이아웃을 강제하는 연산에 대한 자세한 내용은 [this gist](https://gist.github.com/paulirish/5d52fb081b3570c81e3a)와 [CSS Triggers](https://csstriggers.com/)를 참고한다.

### 7.2 Build Index Maps for Repeated Lookups

**Impact: LOW-MEDIUM (1M ops to 2K ops)**

같은 키로 반복되는 `.find()` 호출은 Map을 사용한다.

**Incorrect (O(n) per lookup):**

```typescript
function processOrders(orders: Order[], users: User[]) {
  return orders.map(order => ({
    ...order,
    user: users.find(u => u.id === order.userId)
  }))
}
```

**Correct (O(1) per lookup):**

```typescript
function processOrders(orders: Order[], users: User[]) {
  const userById = new Map(users.map(u => [u.id, u]))

  return orders.map(order => ({
    ...order,
    user: userById.get(order.userId)
  }))
}
```

map을 한 번 만들고 나면(O(n)) 이후 모든 조회는 O(1)이다.

주문 1000개 × 사용자 1000명: 1M ops → 2K ops.

### 7.3 Cache Property Access in Loops

**Impact: LOW-MEDIUM (reduces lookups)**

핫 패스에서 객체 속성 조회를 캐시한다.

**Incorrect: 3 lookups × N iterations**

```typescript
for (let i = 0; i < arr.length; i++) {
  process(obj.config.settings.value)
}
```

**Correct: 1 lookup total**

```typescript
const value = obj.config.settings.value
const len = arr.length
for (let i = 0; i < len; i++) {
  process(value)
}
```

### 7.4 Cache Repeated Function Calls

**Impact: MEDIUM (avoid redundant computation)**

렌더링 중 동일한 입력으로 같은 함수가 반복 호출된다면, 모듈 레벨 Map을 사용해 결과를 캐시한다.

**Incorrect: redundant computation**

```typescript
function ProjectList({ projects }: { projects: Project[] }) {
  return (
    <div>
      {projects.map(project => {
        // slugify() called 100+ times for same project names
        const slug = slugify(project.name)
        
        return <ProjectCard key={project.id} slug={slug} />
      })}
    </div>
  )
}
```

**Correct: cached results**

```typescript
// Module-level cache
const slugifyCache = new Map<string, string>()

function cachedSlugify(text: string): string {
  if (slugifyCache.has(text)) {
    return slugifyCache.get(text)!
  }
  const result = slugify(text)
  slugifyCache.set(text, result)
  return result
}

function ProjectList({ projects }: { projects: Project[] }) {
  return (
    <div>
      {projects.map(project => {
        // Computed only once per unique project name
        const slug = cachedSlugify(project.name)
        
        return <ProjectCard key={project.id} slug={slug} />
      })}
    </div>
  )
}
```

**Simpler pattern for single-value functions:**

```typescript
let isLoggedInCache: boolean | null = null

function isLoggedIn(): boolean {
  if (isLoggedInCache !== null) {
    return isLoggedInCache
  }
  
  isLoggedInCache = document.cookie.includes('auth=')
  return isLoggedInCache
}

// Clear cache when auth changes
function onAuthChange() {
  isLoggedInCache = null
}
```

hook이 아닌 Map을 사용하므로 React 컴포넌트뿐 아니라 유틸리티, 이벤트 핸들러 등 어디서나 동작한다.

Reference: [https://vercel.com/blog/how-we-made-the-vercel-dashboard-twice-as-fast](https://vercel.com/blog/how-we-made-the-vercel-dashboard-twice-as-fast)

### 7.5 Cache Storage API Calls

**Impact: LOW-MEDIUM (reduces expensive I/O)**

`localStorage`, `sessionStorage`, `document.cookie`는 동기적이고 비싸다. 메모리에 읽기 결과를 캐시한다.

**Incorrect: reads storage on every call**

```typescript
function getTheme() {
  return localStorage.getItem('theme') ?? 'light'
}
// Called 10 times = 10 storage reads
```

**Correct: Map cache**

```typescript
const storageCache = new Map<string, string | null>()

function getLocalStorage(key: string) {
  if (!storageCache.has(key)) {
    storageCache.set(key, localStorage.getItem(key))
  }
  return storageCache.get(key)
}

function setLocalStorage(key: string, value: string) {
  localStorage.setItem(key, value)
  storageCache.set(key, value)  // keep cache in sync
}
```

hook이 아닌 Map을 사용하므로 React 컴포넌트뿐 아니라 유틸리티, 이벤트 핸들러 등 어디서나 동작한다.

**Cookie caching:**

```typescript
let cookieCache: Record<string, string> | null = null

function getCookie(name: string) {
  if (!cookieCache) {
    cookieCache = Object.fromEntries(
      document.cookie.split('; ').map(c => c.split('='))
    )
  }
  return cookieCache[name]
}
```

**Important: invalidate on external changes**

```typescript
window.addEventListener('storage', (e) => {
  if (e.key) storageCache.delete(e.key)
})

document.addEventListener('visibilitychange', () => {
  if (document.visibilityState === 'visible') {
    storageCache.clear()
  }
})
```

storage가 외부에서 변경될 수 있다면(다른 탭, 서버에서 설정한 쿠키 등) 캐시를 무효화한다.

### 7.6 Combine Multiple Array Iterations

**Impact: LOW-MEDIUM (reduces iterations)**

여러 번의 `.filter()`나 `.map()` 호출은 배열을 여러 번 순회한다. 하나의 루프로 합친다.

**Incorrect: 3 iterations**

```typescript
const admins = users.filter(u => u.isAdmin)
const testers = users.filter(u => u.isTester)
const inactive = users.filter(u => !u.isActive)
```

**Correct: 1 iteration**

```typescript
const admins: User[] = []
const testers: User[] = []
const inactive: User[] = []

for (const user of users) {
  if (user.isAdmin) admins.push(user)
  if (user.isTester) testers.push(user)
  if (!user.isActive) inactive.push(user)
}
```

### 7.7 Defer Non-Critical Work with requestIdleCallback

**Impact: MEDIUM (keeps UI responsive during background tasks)**

`requestIdleCallback()`을 사용해 비핵심 작업을 브라우저 idle 시간에 스케줄링한다. 메인 스레드를 사용자 상호작용과 애니메이션에 양보해 jank를 줄이고 체감 성능을 개선한다.

**Incorrect: blocks main thread during user interaction**

```typescript
function handleSearch(query: string) {
  const results = searchItems(query)
  setResults(results)

  // These block the main thread immediately
  analytics.track('search', { query })
  saveToRecentSearches(query)
  prefetchTopResults(results.slice(0, 3))
}
```

**Correct: defers non-critical work to idle time**

```typescript
function handleSearch(query: string) {
  const results = searchItems(query)
  setResults(results)

  // Defer non-critical work to idle periods
  requestIdleCallback(() => {
    analytics.track('search', { query })
  })

  requestIdleCallback(() => {
    saveToRecentSearches(query)
  })

  requestIdleCallback(() => {
    prefetchTopResults(results.slice(0, 3))
  })
}
```

**With timeout for required work:**

```typescript
// Ensure analytics fires within 2 seconds even if browser stays busy
requestIdleCallback(
  () => analytics.track('page_view', { path: location.pathname }),
  { timeout: 2000 }
)
```

**Chunking large tasks:**

```typescript
function processLargeDataset(items: Item[]) {
  let index = 0

  function processChunk(deadline: IdleDeadline) {
    // Process items while we have idle time (aim for <50ms chunks)
    while (index < items.length && deadline.timeRemaining() > 0) {
      processItem(items[index])
      index++
    }

    // Schedule next chunk if more items remain
    if (index < items.length) {
      requestIdleCallback(processChunk)
    }
  }

  requestIdleCallback(processChunk)
}
```

**With fallback for unsupported browsers:**

```typescript
const scheduleIdleWork = window.requestIdleCallback ?? ((cb: () => void) => setTimeout(cb, 1))

scheduleIdleWork(() => {
  // Non-critical work
})
```

**When to use:**

- 분석·텔레메트리

- localStorage/IndexedDB에 상태 저장

- 다음 행동에 대비한 리소스 prefetch

- 급하지 않은 데이터 변환

- 비핵심 기능의 lazy 초기화

**When NOT to use:**

- 즉각적인 피드백이 필요한 사용자 동작

- 사용자가 기다리는 렌더링 업데이트

- 시간에 민감한 연산

### 7.8 Early Length Check for Array Comparisons

**Impact: MEDIUM-HIGH (avoids expensive operations when lengths differ)**

배열을 비싼 연산(정렬, 깊은 동등 비교, 직렬화)으로 비교할 때 길이부터 확인한다. 길이가 다르면 같을 수 없다.

실제 애플리케이션에서는 핫 패스(이벤트 핸들러, 렌더 루프)에서 비교가 실행될 때 이 최적화의 가치가 특히 크다.

**Incorrect: always runs expensive comparison**

```typescript
function hasChanges(current: string[], original: string[]) {
  // Always sorts and joins, even when lengths differ
  return current.sort().join() !== original.sort().join()
}
```

`current.length`가 5이고 `original.length`가 100이어도 두 번의 O(n log n) 정렬이 실행된다. 게다가 join하고 문자열 비교까지 추가로 일어난다.

**Correct (O(1) length check first):**

```typescript
function hasChanges(current: string[], original: string[]) {
  // Early return if lengths differ
  if (current.length !== original.length) {
    return true
  }
  // Only sort when lengths match
  const currentSorted = current.toSorted()
  const originalSorted = original.toSorted()
  for (let i = 0; i < currentSorted.length; i++) {
    if (currentSorted[i] !== originalSorted[i]) {
      return true
    }
  }
  return false
}
```

이 새 접근이 더 효율적인 이유는 다음과 같다.

- 길이가 다를 때 정렬·join 오버헤드를 피한다

- join된 문자열에 대한 메모리 소비를 피한다 (큰 배열에서 특히 중요)

- 원본 배열을 변경하지 않는다

- 차이를 발견하면 즉시 return한다

### 7.9 Early Return from Functions

**Impact: LOW-MEDIUM (avoids unnecessary computation)**

결과가 결정되면 즉시 return해서 불필요한 처리를 건너뛴다.

**Incorrect: processes all items even after finding answer**

```typescript
function validateUsers(users: User[]) {
  let hasError = false
  let errorMessage = ''
  
  for (const user of users) {
    if (!user.email) {
      hasError = true
      errorMessage = 'Email required'
    }
    if (!user.name) {
      hasError = true
      errorMessage = 'Name required'
    }
    // Continues checking all users even after error found
  }
  
  return hasError ? { valid: false, error: errorMessage } : { valid: true }
}
```

**Correct: returns immediately on first error**

```typescript
function validateUsers(users: User[]) {
  for (const user of users) {
    if (!user.email) {
      return { valid: false, error: 'Email required' }
    }
    if (!user.name) {
      return { valid: false, error: 'Name required' }
    }
  }

  return { valid: true }
}
```

### 7.10 Hoist RegExp Creation

**Impact: LOW-MEDIUM (avoids recreation)**

render 안에서 RegExp를 생성하지 않는다. 모듈 스코프로 끌어올리거나 `useMemo()`로 메모이제이션한다.

**Incorrect: new RegExp every render**

```tsx
function Highlighter({ text, query }: Props) {
  const regex = new RegExp(`(${query})`, 'gi')
  const parts = text.split(regex)
  return <>{parts.map((part, i) => ...)}</>
}
```

**Correct: memoize or hoist**

```tsx
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

function Highlighter({ text, query }: Props) {
  const regex = useMemo(
    () => new RegExp(`(${escapeRegex(query)})`, 'gi'),
    [query]
  )
  const parts = text.split(regex)
  return <>{parts.map((part, i) => ...)}</>
}
```

**Warning: global regex has mutable state**

```typescript
const regex = /foo/g
regex.test('foo')  // true, lastIndex = 3
regex.test('foo')  // false, lastIndex = 0
```

global regex(`/g`)는 변경 가능한 `lastIndex` 상태를 가진다.

### 7.11 Use flatMap to Map and Filter in One Pass

**Impact: LOW-MEDIUM (eliminates intermediate array)**

`.map().filter(Boolean)` 체인은 중간 배열을 만들고 두 번 순회한다. `.flatMap()`을 사용해 한 번의 순회로 변환과 필터링을 동시에 한다.

**Incorrect: 2 iterations, intermediate array**

```typescript
const userNames = users
  .map(user => user.isActive ? user.name : null)
  .filter(Boolean)
```

**Correct: 1 iteration, no intermediate array**

```typescript
const userNames = users.flatMap(user =>
  user.isActive ? [user.name] : []
)
```

**More examples:**

```typescript
// Extract valid emails from responses
// Before
const emails = responses
  .map(r => r.success ? r.data.email : null)
  .filter(Boolean)

// After
const emails = responses.flatMap(r =>
  r.success ? [r.data.email] : []
)

// Parse and filter valid numbers
// Before
const numbers = strings
  .map(s => parseInt(s, 10))
  .filter(n => !isNaN(n))

// After
const numbers = strings.flatMap(s => {
  const n = parseInt(s, 10)
  return isNaN(n) ? [] : [n]
})
```

**When to use:**

- 일부를 걸러내면서 나머지를 변환할 때

- 일부 입력은 출력이 없는 조건부 매핑

- 잘못된 입력은 건너뛰는 파싱·검증

### 7.12 Use Loop for Min/Max Instead of Sort

**Impact: LOW (O(n) instead of O(n log n))**

최소·최대 원소를 찾는 데는 한 번의 순회만 필요하다. 정렬은 낭비이고 더 느리다.

**Incorrect (O(n log n) - sort to find latest):**

```typescript
interface Project {
  id: string
  name: string
  updatedAt: number
}

function getLatestProject(projects: Project[]) {
  const sorted = [...projects].sort((a, b) => b.updatedAt - a.updatedAt)
  return sorted[0]
}
```

최댓값 하나를 구하기 위해 배열 전체를 정렬한다.

**Incorrect (O(n log n) - sort for oldest and newest):**

```typescript
function getOldestAndNewest(projects: Project[]) {
  const sorted = [...projects].sort((a, b) => a.updatedAt - b.updatedAt)
  return { oldest: sorted[0], newest: sorted[sorted.length - 1] }
}
```

min/max만 필요한데도 여전히 불필요하게 정렬한다.

**Correct (O(n) - single loop):**

```typescript
function getLatestProject(projects: Project[]) {
  if (projects.length === 0) return null
  
  let latest = projects[0]
  
  for (let i = 1; i < projects.length; i++) {
    if (projects[i].updatedAt > latest.updatedAt) {
      latest = projects[i]
    }
  }
  
  return latest
}

function getOldestAndNewest(projects: Project[]) {
  if (projects.length === 0) return { oldest: null, newest: null }
  
  let oldest = projects[0]
  let newest = projects[0]
  
  for (let i = 1; i < projects.length; i++) {
    if (projects[i].updatedAt < oldest.updatedAt) oldest = projects[i]
    if (projects[i].updatedAt > newest.updatedAt) newest = projects[i]
  }
  
  return { oldest, newest }
}
```

배열을 한 번만 순회하고, 복사도 정렬도 없다.

**Alternative: Math.min/Math.max for small arrays**

```typescript
const numbers = [5, 2, 8, 1, 9]
const min = Math.min(...numbers)
const max = Math.max(...numbers)
```

작은 배열에는 잘 동작하지만, spread operator의 한계 때문에 매우 큰 배열에서는 느려지거나 에러가 던져질 수 있다. 대략 Chrome 143에서 124,000개, Safari 18에서 638,000개가 한계이며 정확한 값은 환경마다 다르다 — [the fiddle](https://jsfiddle.net/qw1jabsx/4/) 참고. 안정성을 위해 loop 방식을 사용한다.

### 7.13 Use Set/Map for O(1) Lookups

**Impact: LOW-MEDIUM (O(n) to O(1))**

반복적인 멤버십 체크는 배열 대신 Set/Map으로 바꾼다.

**Incorrect (O(n) per check):**

```typescript
const allowedIds = ['a', 'b', 'c', ...]
items.filter(item => allowedIds.includes(item.id))
```

**Correct (O(1) per check):**

```typescript
const allowedIds = new Set(['a', 'b', 'c', ...])
items.filter(item => allowedIds.has(item.id))
```

### 7.14 Use toSorted() Instead of sort() for Immutability

**Impact: MEDIUM-HIGH (prevents mutation bugs in React state)**

`.sort()`는 배열을 in-place로 변경하므로 React의 state·props에서 버그를 유발할 수 있다. `.toSorted()`를 사용해 변형 없이 새 정렬 배열을 만든다.

**Incorrect: mutates original array**

```typescript
function UserList({ users }: { users: User[] }) {
  // Mutates the users prop array!
  const sorted = useMemo(
    () => users.sort((a, b) => a.name.localeCompare(b.name)),
    [users]
  )
  return <div>{sorted.map(renderUser)}</div>
}
```

**Correct: creates new array**

```typescript
function UserList({ users }: { users: User[] }) {
  // Creates new sorted array, original unchanged
  const sorted = useMemo(
    () => users.toSorted((a, b) => a.name.localeCompare(b.name)),
    [users]
  )
  return <div>{sorted.map(renderUser)}</div>
}
```

**Why this matters in React:**

1. props·state 변경은 React의 불변성 모델을 깨뜨린다 — React는 props와 state를 read-only로 취급한다고 가정한다

2. stale closure 버그를 유발한다 — closure(callback, effect) 내부에서 배열을 변경하면 예측하지 못한 동작으로 이어질 수 있다

**Browser support: fallback for older browsers**

```typescript
// Fallback for older browsers
const sorted = [...items].sort((a, b) => a.value - b.value)
```

`.toSorted()`는 모든 최신 브라우저에서 사용 가능하다(Chrome 110+, Safari 16+, Firefox 115+, Node.js 20+). 구형 환경에서는 spread operator를 사용한다.

**Other immutable array methods:**

- `.toSorted()` — 불변 정렬

- `.toReversed()` — 불변 reverse

- `.toSpliced()` — 불변 splice

- `.with()` — 불변 원소 교체

---

## 8. Advanced Patterns

**Impact: LOW**

신중한 구현이 필요한 특정 케이스를 위한 고급 패턴이다.

### 8.1 Initialize App Once, Not Per Mount

**Impact: LOW-MEDIUM (avoids duplicate init in development)**

앱 로드 시 한 번만 실행되어야 하는 앱 전역 초기화를 컴포넌트의 `useEffect([])` 안에 두지 않는다. 컴포넌트는 다시 마운트될 수 있고 effect도 재실행되기 때문이다. 모듈 레벨 가드나 엔트리 모듈의 최상위 초기화를 사용한다.

**Incorrect: runs twice in dev, re-runs on remount**

```tsx
function Comp() {
  useEffect(() => {
    loadFromStorage()
    checkAuthToken()
  }, [])

  // ...
}
```

**Correct: once per app load**

```tsx
let didInit = false

function Comp() {
  useEffect(() => {
    if (didInit) return
    didInit = true
    loadFromStorage()
    checkAuthToken()
  }, [])

  // ...
}
```

Reference: [https://react.dev/learn/you-might-not-need-an-effect#initializing-the-application](https://react.dev/learn/you-might-not-need-an-effect#initializing-the-application)

### 8.2 Store Event Handlers in Refs

**Impact: LOW (stable subscriptions)**

콜백 변경 시 다시 구독되어선 안 되는 effect에서 사용한다면 콜백을 ref에 저장한다.

**Incorrect: re-subscribes on every render**

```tsx
function useWindowEvent(event: string, handler: (e) => void) {
  useEffect(() => {
    window.addEventListener(event, handler)
    return () => window.removeEventListener(event, handler)
  }, [event, handler])
}
```

**Correct: stable subscription**

```tsx
import { useEffectEvent } from 'react'

function useWindowEvent(event: string, handler: (e) => void) {
  const onEvent = useEffectEvent(handler)

  useEffect(() => {
    window.addEventListener(event, onEvent)
    return () => window.removeEventListener(event, onEvent)
  }, [event])
}
```

**Alternative: 최신 React라면 `useEffectEvent`를 사용한다:**

`useEffectEvent`는 동일한 패턴을 더 깔끔한 API로 제공한다. 즉, 항상 최신 버전의 핸들러를 호출하는 안정된 함수 참조를 만들어 준다.

### 8.3 useEffectEvent for Stable Callback Refs

**Impact: LOW (prevents effect re-runs)**

콜백을 의존성 배열에 추가하지 않고도 콜백 안에서 최신 값에 접근한다. effect 재실행을 막으면서 stale closure도 피한다.

**Incorrect: effect re-runs on every callback change**

```tsx
function SearchInput({ onSearch }: { onSearch: (q: string) => void }) {
  const [query, setQuery] = useState('')

  useEffect(() => {
    const timeout = setTimeout(() => onSearch(query), 300)
    return () => clearTimeout(timeout)
  }, [query, onSearch])
}
```

**Correct: using React's useEffectEvent**

```tsx
import { useEffectEvent } from 'react';

function SearchInput({ onSearch }: { onSearch: (q: string) => void }) {
  const [query, setQuery] = useState('')
  const onSearchEvent = useEffectEvent(onSearch)

  useEffect(() => {
    const timeout = setTimeout(() => onSearchEvent(query), 300)
    return () => clearTimeout(timeout)
  }, [query])
}
```

---

## References

1. [https://react.dev](https://react.dev)
2. [https://nextjs.org](https://nextjs.org)
3. [https://swr.vercel.app](https://swr.vercel.app)
4. [https://github.com/shuding/better-all](https://github.com/shuding/better-all)
5. [https://github.com/isaacs/node-lru-cache](https://github.com/isaacs/node-lru-cache)
6. [https://vercel.com/blog/how-we-optimized-package-imports-in-next-js](https://vercel.com/blog/how-we-optimized-package-imports-in-next-js)
7. [https://vercel.com/blog/how-we-made-the-vercel-dashboard-twice-as-fast](https://vercel.com/blog/how-we-made-the-vercel-dashboard-twice-as-fast)
