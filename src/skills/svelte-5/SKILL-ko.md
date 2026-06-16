---
name: svelte-5
description: Svelte 5 + SvelteKit 2 with runes ($state/$derived/$effect/$props/$bindable), snippets (no slots), callback events (no createEventDispatcher), routing, load functions, form actions, and v4→v5 migration. 스벨트 5 컴포넌트, SvelteKit 라우팅, runes 마이그레이션.
keywords: [svelte, svelte-5, 스벨트, sveltekit, sveltekit-2, 스벨트킷, runes, $state, $derived, $effect, $props, $bindable, snippet, load, form-actions, hooks, migration, 마이그레이션]
---

# Svelte 5

관용적인 Svelte 5 컴포넌트와 SvelteKit 2 라우트를 만든다. 기본은 **runes mode**(현재 Svelte 5 기본값), `{@render children()}` / snippet 패턴, callback-prop 이벤트다. 사용자가 명시적으로 5 이전 코드베이스를 유지보수하는 게 아니라면 절대 legacy `export let` / `on:event` / `createEventDispatcher`로 회귀하지 않는다.

작업 시 해당하는 reference 파일을 읽는다:

- [references/runes.md](references/runes.md) — `$state`, `$derived`,
  `$effect`, `$props`, `$bindable`, `$inspect`의 동작 예제와 각 rune이 부적절한 경우
- [references/sveltekit.md](references/sveltekit.md) — routing, `load`
  function (universal vs server), form action + `use:enhance`,
  hooks, `$env`, `$app/state`
- [references/migrating-from-svelte-4.md](references/migrating-from-svelte-4.md)
  — `export let`, `$:`, slot, `createEventDispatcher`, `on:event`,
  `<svelte:component>`을 v5로 기계적으로 옮기는 방법

## 기본 가정

사용자는 다음을 사용한다고 가정한다:

- **Svelte 5** (runes mode, 2024 출시) — `svelte@5` 이후 기본값.
- **SvelteKit 2** — `+page.svelte`, `+layout.ts`, `+page.server.ts`
  파일 시스템 라우팅; form action; `$app/state` (`$app/stores`가 아님).
- **TypeScript** — 사용자 파일이 명시적으로 plain JS를 쓰지 않는 한.

확신이 안 설 때는 모던 경로를 택한다 — store보다 rune, slot보다 snippet, dispatcher보다 callback prop.

## Runes 한눈에 보기

| Rune | Role | When to use |
|------|------|-------------|
| `$state(x)` | 깊은 proxy를 가진 reactive 값 | 시간에 따라 변하면서 UI를 구동하는 모든 값 |
| `$state.raw(x)` | 얕음 — proxy 미적용 | 깊은 proxy가 낭비인 큰 immutable 구조 |
| `$derived(expr)` | 의존성 변화 시 재실행되는 순수 computed 값 | 다른 reactive state로부터의 expression |
| `$derived.by(() => ...)` | 함수를 받는 `$derived` | 다중 statement body가 필요한 derived 값 |
| `$effect(() => ...)` | 의존성 변화 시 재실행되는 side effect | DOM, subscription, analytics, canvas — **state 업데이트 아님** |
| `$effect.pre(() => ...)` | DOM commit 이전 effect | paint 전 측정/layout read |
| `$props()` | 컴포넌트 props | 입력을 받는 모든 컴포넌트 |
| `$bindable(default?)` | 양방향 바인딩 opt-in 슬롯 | 부모가 `bind:value`를 쓰고 싶은 form 래퍼 |
| `$inspect(...)` | 변경 시 dev 전용 log | 반응성 디버깅 |

자세한 내용, 함정, 예제: [runes.md](references/runes.md).

## 컴포넌트 형태

Svelte 5 컴포넌트는 세 개의 순차 슬롯을 가진 `.svelte` 파일이다:

```svelte
<script lang="ts">
  // 1. Imports
  import type { Snippet } from 'svelte'
  import { slide } from 'svelte/transition'

  // 2. Props (always $props, destructured, typed)
  type Props = {
    title: string
    open?: boolean
    children?: Snippet        // use Snippet from 'svelte', not a function type
  }
  let { title, open = false, children }: Props = $props()

  // 3. Local reactive state + derived + effects
  let expanded = $state(open)
  const caret = $derived(expanded ? '▾' : '▸')

  $effect(() => {
    // DOM / subscription / log side effects only; never assign reactive state here
    console.log('expanded:', expanded)
  })

  function toggle() { expanded = !expanded }
</script>

<button onclick={toggle} aria-expanded={expanded}>
  {caret} {title}
</button>

{#if expanded}
  <div transition:slide>
    {@render children?.()}
  </div>
{/if}

<style>
  button { font-weight: 600; }
</style>
```

핵심 결정사항:

- **Props는 `$props()`**, `export let` 아님. Destructure하고 typing.
- **자식은 `Snippet`**이며 `{@render children?.()}`로 렌더한다 — `<slot>` 아님.
- **이벤트는 attribute** (`onclick`, `oninput`) — `on:click` 아님. `createEventDispatcher` 없음; callback prop으로 발신 (`onclose`, `onsubmit` 등).
- **Transition은 그대로** — `svelte/transition`과 `svelte/motion`이 여전히 올바른 import.

복사해서 쓸 수 있는 전체 템플릿: [assets/component.svelte](assets/component.svelte).

## 반응형 state 공유 (`.svelte.ts`)

Rune은 `.svelte`, `.svelte.js`, `.svelte.ts` 파일 안에서 모두 동작한다. 여러 컴포넌트가 같은 reactive source를 필요로 하면 `.svelte.ts` 모듈을 사용한다. **신규 코드에 legacy `writable`/`readable` store를 사용하지 말 것** — 이들은 backwards compatibility만을 위해 동작한다.

```ts
// cart.svelte.ts
type Item = { id: string; name: string; qty: number }

let items = $state<Item[]>([])
const total = $derived(items.reduce((n, it) => n + it.qty, 0))

export const cart = {
  get items()  { return items },
  get total()  { return total },
  add(item: Item)    { items.push(item) },
  remove(id: string) { items = items.filter((it) => it.id !== id) },
  clear()            { items = [] },
}
```

이후 어떤 컴포넌트에서든: `import { cart } from './cart.svelte.ts'`. `cart.items`와 `cart.total`은 컴포넌트 경계를 넘어서도 reactive하게 유지된다. 전체 패턴: [assets/store.svelte.ts](assets/store.svelte.ts).

## SvelteKit 2 — 파일 역할

| File | Runs on | Purpose |
|------|---------|---------|
| `+page.svelte` | Client (SSR로부터 hydrate됨) | 페이지 UI |
| `+page.ts` | Universal (server + client) | `load`; 어느 환경에서든 안전 |
| `+page.server.ts` | **Server only** | `load`, `actions`, secret/DB 접근 |
| `+layout.svelte` | Client | 공유 wrapping UI (nav, sidebar) |
| `+layout.ts` / `+layout.server.ts` | 위와 동일 | 자식 라우트용 공유 데이터 |
| `+error.svelte` | Client | `load`가 throw 했을 때 fallback UI |
| `+server.ts` | Server | REST 스타일 API 엔드포인트 (`GET`, `POST`, ...) |
| `hooks.server.ts` | Server | `handle`, `handleFetch`, `handleError` |
| `hooks.client.ts` | Client | `handleError`의 클라이언트 측 |

전체 routing + load + action 흐름: [sveltekit.md](references/sveltekit.md).

## Load 함수 — universal vs server

```ts
// +page.ts  (universal — runs on server first, then client on nav)
import type { PageLoad } from './$types'
export const load: PageLoad = async ({ fetch, params }) => {
  const res = await fetch(`/api/posts/${params.slug}`)   // use the scoped fetch
  if (!res.ok) throw error(404, 'Post not found')
  return { post: await res.json() }
}
```

```ts
// +page.server.ts  (server-only — access secrets, DB)
import type { PageServerLoad } from './$types'
import { env } from '$env/dynamic/private'
export const load: PageServerLoad = async ({ locals, params }) => {
  const post = await locals.db.posts.findUnique({ where: { slug: params.slug } })
  return { post, revalidateSecret: env.REVALIDATE_SECRET }    // never leak: pick fields
}
```

가이드라인:

- 엔드포인트가 public이고 클라이언트 측 nav가 server round-trip을 건너뛰길 원하면 `+page.ts`를 쓴다.
- 데이터 소스가 server-only(DB, secret, filesystem)이면 `+page.server.ts`를 쓴다.
- universal 파일에서 `$env/static/private` 또는 `$env/dynamic/private`을 import 하지 않는다 — 컴파일러가 정당한 이유로 막는다.

## Form 액션 — mutation의 첫 번째 도구

```ts
// +page.server.ts
import type { Actions, PageServerLoad } from './$types'
import { fail, redirect } from '@sveltejs/kit'

export const actions: Actions = {
  default: async ({ request, locals }) => {
    const data = await request.formData()
    const title = String(data.get('title') ?? '').trim()
    if (!title) return fail(400, { title, error: 'Title is required' })
    const post = await locals.db.posts.create({ data: { title } })
    redirect(303, `/posts/${post.slug}`)
  },
}
```

```svelte
<!-- +page.svelte -->
<script lang="ts">
  import { enhance } from '$app/forms'
  import type { ActionData } from './$types'
  let { form }: { form: ActionData } = $props()
</script>

<form method="POST" use:enhance>
  <input name="title" value={form?.title ?? ''} />
  {#if form?.error}<p class="err">{form.error}</p>{/if}
  <button>Create</button>
</form>
```

`use:enhance`는 progressive enhancement(JS가 비활성화돼도 동작)와 응답에 대한 fine-grained control을 제공한다. 페이지에 속하는 mutation에는 `fetch('/api/...', { method: 'POST' })`보다 form action을 선호한다.

## 이벤트 — dispatcher가 아닌 callback prop

```svelte
<!-- Modal.svelte -->
<script lang="ts">
  let { onclose }: { onclose: () => void } = $props()
</script>

<button onclick={onclose}>Close</button>

<!-- usage -->
<Modal onclose={() => (modalOpen = false)} />
```

`createEventDispatcher` 없음. 부모가 함수를 넘기고 자식이 호출한다. 런타임에서 더 저렴하고, 완전 typed이며, 어떤 callback shape(단일 인자, 다중 인자, promise 반환)에도 동작한다.

## Snippets — slot의 대체

```svelte
<!-- List.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte'
  type Props = {
    items: string[]
    row: Snippet<[item: string, index: number]>
  }
  let { items, row }: Props = $props()
</script>

<ul>
  {#each items as item, i}
    <li>{@render row(item, i)}</li>
  {/each}
</ul>

<!-- usage -->
<List items={['a', 'b']}>
  {#snippet row(item, i)}
    <span>{i}: {item}</span>
  {/snippet}
</List>
```

Snippet은 (slot과 달리) 위치 인자를 받고, `Snippet<[Args]>`로 typing되며, 어디든 prop으로 전달 가능하다. props를 받는 slot이었던 모든 것에 사용한다.

## Svelte 4에서의 마이그레이션

사용자가 v4 코드를 가지고 있다면, 기계적 번역:

| v4 | v5 |
|----|----|
| `export let name` | `let { name } = $props()` |
| `let count = 0`<br>`$: doubled = count * 2` | `let count = $state(0)`<br>`const doubled = $derived(count * 2)` |
| `$: if (x > 10) warn()` | `$effect(() => { if (x > 10) warn() })` |
| `on:click={handler}` | `onclick={handler}` |
| `<slot />` | `{@render children?.()}` + props에 `children?: Snippet` |
| `<slot name="foo" />` | named-prop snippet: `foo?: Snippet` + `{@render foo?.()}` |
| `createEventDispatcher()` + `dispatch('save')` | `onsave` callback prop |
| `<svelte:component this={Comp} />` | `<Comp />` (v5에서 컴포넌트는 값) |
| `.ts`의 `writable(x)` | `.svelte.ts`의 `let v = $state(x)` |

자세한 전환 함정 (reactive loop, effect timing, legacy mode): [migrating-from-svelte-4.md](references/migrating-from-svelte-4.md).

## 안티패턴

- **`$effect` 안에서 reactive state에 쓰지 말 것** — 재실행 루프를 일으킨다. 업데이트는 그것을 트리거한 이벤트 핸들러에 둔다. 진정한 cascade가 필요하면 `$derived`(순수)를 쓴다.
- **신규 코드에서 `svelte/store`에 손을 뻗지 말 것** — `.svelte.ts` + `$state`로 subscribe/unsubscribe 없이 cross-component 공유를 다 한다.
- **SvelteKit 2 코드에서 `$app/stores`를 쓰지 말 것** — `$app/state`가 권장되며 deprecated. `page.url`, `page.params`, `page.status`는 이제 plain reactive 프로퍼티이고 `$page` prefix가 없다.
- **`export let`과 `$props()`를 섞지 말 것** — 컴파일러가 경고하고 그 변수에 대해 컴포넌트가 rune 기능에서 잠긴다.
- **reactive attribute로 충분할 때 imperative DOM property 설정 금지** — `element.classList.add(...)`를 호출하는 `$effect`는 보통 `class={{ ... }}` shorthand가 어울린다.

## 통합 노트

- 테스트 — snippet을 지원하는 `vitest` + `@testing-library/svelte` v5+ 사용. SvelteKit은 `playwright`로 e2e 커버.
- 스타일링 — 기본은 scoped `<style>`; escape는 `:global(...)`. Tailwind / UnoCSS는 `sv add tailwindcss`로 통합.
- 타입 생성 — SvelteKit은 라우트별로 `./$types`를 자동 생성한다 (`PageLoad`, `PageServerLoad`, `Actions`, `PageData`).
