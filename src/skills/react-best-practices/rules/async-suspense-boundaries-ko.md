---
title: Strategic Suspense Boundaries
impact: HIGH
impactDescription: faster initial paint
tags: async, suspense, streaming, layout-shift
---

## Strategic Suspense Boundaries

async 컴포넌트에서 JSX를 반환하기 전에 데이터를 await하지 말고, Suspense 경계를 사용해 데이터가 로드되는 동안 외곽 UI를 먼저 보여준다.

**Incorrect (wrapper blocked by data fetching):**

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

**Correct (wrapper shows immediately, data streams in):**

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

**Alternative (share promise across components):**

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
