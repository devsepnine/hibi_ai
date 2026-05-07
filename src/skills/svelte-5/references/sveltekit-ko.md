# SvelteKit 2

SvelteKit은 Svelte의 공식 풀스택 프레임워크이다. 이 레퍼런스는 실제 앱에 필요한 것을 다룬다: 라우팅, load 함수, form action, hook, 환경 변수, v2에서 store를 대체한 `$app/state` API.

## File-system routing

라우트는 `src/routes/` 아래에 산다. 각 폴더는 URL 세그먼트이다. 특수 파일명이 동작을 제어한다:

```
src/routes/
├── +page.svelte                       # GET / (renders page UI)
├── +page.ts                           # universal load for /
├── +layout.svelte                     # shared UI around children
├── +layout.ts                         # shared universal load
├── +error.svelte                      # shown when a load throws
├── blog/
│   ├── +page.svelte                   # GET /blog
│   ├── +page.server.ts                # server-only load / actions
│   └── [slug]/
│       └── +page.svelte               # GET /blog/:slug
├── api/
│   └── posts/
│       └── +server.ts                 # GET/POST /api/posts (JSON)
└── (marketing)/                       # route group — no URL segment
    ├── +layout.svelte
    └── about/+page.svelte             # GET /about (inherits marketing layout)
```

### Dynamic and optional segments

- `[slug]` — required: `/blog/hello`
- `[[optional]]` — optional: `/blog` 또는 `/blog/hello`
- `[...rest]` — catch-all: `/docs/a/b/c`
- `[param=matcher]` — validated: `params.*.ts` matcher 참조

### Route groups `(name)`

괄호로 감싼 폴더는 URL 세그먼트가 되지 않는다. URL을 변경하지 않고 라우트의 부분집합에 자체 layout을 주기 위해 사용한다.

## Load functions — universal vs server

### Universal — `+page.ts`, `+layout.ts`

초기 렌더링에서는 서버에서 실행되고, 클라이언트 사이드 nav에서는 브라우저에서 실행된다. 데이터 소스가 공개 endpoint일 때 사용한다.

```ts
// src/routes/blog/[slug]/+page.ts
import { error } from '@sveltejs/kit'
import type { PageLoad } from './$types'

export const load: PageLoad = async ({ fetch, params }) => {
  const res = await fetch(`/api/posts/${params.slug}`)   // scoped fetch
  if (!res.ok) error(404, 'Not found')
  return { post: await res.json() }
}
```

핵심 포인트:

- 글로벌이 **아닌** `fetch` 인자를 사용한다 — 쿠키를 보존하고, 상대 URL을 해석하며, 서버에서 내부 라우트의 네트워크 hop을 피한다.
- `error(...)`를 throw하면 가장 가까운 `+error.svelte`를 트리거한다.

### Server — `+page.server.ts`, `+layout.server.ts`

서버에서만 실행된다. DB 액세스, private env var, 파일시스템 read, auth 체크에 필요하다.

```ts
// +page.server.ts
import type { PageServerLoad } from './$types'

export const load: PageServerLoad = async ({ locals, params }) => {
  const post = await locals.db.posts.findUnique({ where: { slug: params.slug } })
  if (!post) error(404, 'Not found')
  return { post }           // ships to the client as serialized JSON
}
```

규칙:

- universal (`.ts`, `.server.ts` 아닌) 파일에서 `$env/static/private` 또는 `$env/dynamic/private`을 절대 import하지 마라 — 컴파일러가 막는다.
- 반환된 객체는 직렬화된다. function, class instance, proxy는 strip된다. 평이한 데이터를 반환한다.
- DB connection 같은 request-scoped 서비스는 `locals` (handle hook에서 채워짐)를 사용한다.

### Consuming load data

모든 `+page.svelte` / `+layout.svelte`는 병합된 load 출력을 `data`로 받는다:

```svelte
<script lang="ts">
  import type { PageData } from './$types'
  let { data }: { data: PageData } = $props()
</script>

<h1>{data.post.title}</h1>
```

`PageData`는 `load` 반환 타입에서 생성된다 — 수동 타이핑 없음.

### Invalidation

내비게이션 없이 데이터를 새로고침해야 할 때:

```ts
import { invalidate, invalidateAll } from '$app/navigation'
await invalidate('app:posts')        // rerun loads that opted in via `depends('app:posts')`
await invalidateAll()                 // rerun every load on the current route
```

load 내부에서:

```ts
export const load: PageLoad = async ({ depends, fetch }) => {
  depends('app:posts')
  return { posts: await (await fetch('/api/posts')).json() }
}
```

## Form actions — the default mutation path

사용자 주도 변경 (create/update/delete)은 form action으로 시작한다. JS 없이 작동하고, progressive enhancement를 위해 `use:enhance`와 통합되며, 타이핑을 빈틈없이 유지한다.

```ts
// +page.server.ts
import type { Actions, PageServerLoad } from './$types'
import { fail, redirect } from '@sveltejs/kit'

export const load: PageServerLoad = async ({ locals }) => ({
  posts: await locals.db.posts.findMany(),
})

export const actions: Actions = {
  // default action: <form method="POST">
  default: async ({ request, locals }) => {
    const data = await request.formData()
    const title = String(data.get('title') ?? '').trim()
    if (!title) return fail(400, { title, error: 'Title is required' })

    const post = await locals.db.posts.create({ data: { title } })
    redirect(303, `/posts/${post.slug}`)
  },

  // named action: <form method="POST" action="?/delete">
  delete: async ({ request, locals }) => {
    const data = await request.formData()
    await locals.db.posts.delete({ where: { id: String(data.get('id')) } })
    return { deleted: true }
  },
}
```

```svelte
<!-- +page.svelte -->
<script lang="ts">
  import { enhance } from '$app/forms'
  import type { ActionData, PageData } from './$types'
  let { data, form }: { data: PageData; form: ActionData } = $props()
</script>

<form method="POST" use:enhance>
  <input name="title" value={form?.title ?? ''} />
  {#if form?.error}<p class="err">{form.error}</p>{/if}
  <button>Create</button>
</form>

<ul>
  {#each data.posts as p (p.id)}
    <li>
      {p.title}
      <form method="POST" action="?/delete" use:enhance>
        <input type="hidden" name="id" value={p.id} />
        <button>Delete</button>
      </form>
    </li>
  {/each}
</ul>
```

### `fail` vs `error` vs `redirect`

- `fail(status, data)` — validation 실패; 데이터를 form으로 다시 반환하여 UI가 에러와 함께 다시 렌더링한다. 같은 페이지에 머무른다.
- `error(status, message)` — 복구 불가능; `+error.svelte`를 렌더링한다.
- `redirect(303, path)` — 성공한 mutation; 새 URL로 보낸다 (종종 생성/업데이트된 리소스).

### Custom enhance behavior

```svelte
<form method="POST" use:enhance={() => async ({ update, result }) => {
  // Run before SvelteKit applies the result
  if (result.type === 'success') confetti()
  await update()          // apply the result (this is what default enhance does)
}}>
```

성공/실패에 사이드 이펙트가 필요할 때 (toast, confetti, optimistic UI) 커스텀 enhance를 사용한다. 대부분의 폼에는 default enhance가 괜찮다.

## Hooks — request middleware

### `hooks.server.ts`

```ts
import type { Handle, HandleServerError } from '@sveltejs/kit'
import { sequence } from '@sveltejs/kit/hooks'
import { db } from '$lib/server/db'

const authHandle: Handle = async ({ event, resolve }) => {
  const token = event.cookies.get('session')
  event.locals.user = token ? await db.session.getUser(token) : null
  return resolve(event)
}

const injectDb: Handle = async ({ event, resolve }) => {
  event.locals.db = db
  return resolve(event)
}

export const handle: Handle = sequence(injectDb, authHandle)

export const handleError: HandleServerError = ({ error, event }) => {
  // log, report to Sentry, etc. Return a shape the client can read.
  return { message: 'Internal error', code: 'INTERNAL' }
}
```

- `event.locals`는 request-scoped 데이터를 attach하는 단일 장소이다. `load`와 `+server.ts` 모두가 read할 수 있도록 `handle`에서 채운다.
- `sequence(...)`는 여러 핸들러를 조합한다.
- `handleError`는 서버 사이드 에러 트랩이다. 객체를 반환한다; 그것이 `$app/state`의 `page.error`가 된다.

### `hooks.client.ts`

표면이 작다 — 대부분 클라이언트 사이드 에러를 위한 `handleError`.

```ts
import type { HandleClientError } from '@sveltejs/kit'

export const handleError: HandleClientError = ({ error }) => {
  console.error(error)
  return { message: 'Something went wrong' }
}
```

## Environment variables

static/dynamic × public/private에 따라 선택되는 네 가지:

| Module | Safe in client? | Reads change at runtime? |
|--------|:---:|:---:|
| `$env/static/public`  | ✅ | ❌ (build time에 baked) |
| `$env/static/private` | ❌ | ❌ |
| `$env/dynamic/public`  | ✅ | ✅ |
| `$env/dynamic/private` | ❌ | ✅ |

```ts
import { PUBLIC_API_URL }    from '$env/static/public'   // e.g. PUBLIC_API_URL in .env
import { env as priv }       from '$env/dynamic/private' // process.env at runtime
```

규칙:

- `*/public`의 모든 것에 `PUBLIC_*` prefix가 필요하다.
- 브라우저로 ship되는 파일에서 `*/private`을 import하면 컴파일 에러이다.
- `static/*`을 선호한다 — 더 나은 tree-shaking과 런타임 lookup 비용 없음. 값이 실제로 deploy/요청별로 변경될 때만 `dynamic/*`을 사용한다.

## `$app/state` — replaced `$app/stores` in v2

```ts
import { page, navigating, updated } from '$app/state'

// Plain reactive access — no $page prefix
console.log(page.url.pathname, page.params, page.status, page.error)

if (navigating) {
  console.log('going to', navigating.to?.url.pathname)
}
```

코드에서 `$app/stores`나 `$page`가 보인다면 v1/v2-early 패턴이다 — 마이그레이션한다. v2 `$app/state` API는 이미 runes로 추적되므로 그저 field를 read하면 된다.

## API endpoints — `+server.ts`

JSON-ish REST endpoint (모바일 클라이언트, webhook, 페이지를 렌더링하지 않는 것):

```ts
// src/routes/api/posts/+server.ts
import { json } from '@sveltejs/kit'
import type { RequestHandler } from './$types'

export const GET: RequestHandler = async ({ locals }) => {
  const posts = await locals.db.posts.findMany()
  return json(posts)
}

export const POST: RequestHandler = async ({ request, locals }) => {
  const { title } = await request.json()
  const post = await locals.db.posts.create({ data: { title } })
  return json(post, { status: 201 })
}
```

페이지 소유 mutation에는 form action을 선호한다; cross-client API에는 `+server.ts`를 사용한다.

## Page options

`+page.ts` / `+layout.ts`의 파일별 export:

```ts
export const prerender = true       // emit as static HTML at build
export const ssr = false             // SPA: skip server rendering
export const csr = true              // keep client hydration (default)
export const trailingSlash = 'never' // url normalization
```

진정으로 정적인 페이지 (마케팅, 문서)에 `prerender = true`를 사용한다 — deploy 크기와 cold-start 비용을 줄인다. 풀 SSG 사이트는 `adapter-static`과 결합한다.

## Adapters (quick map)

`svelte.config.js`로 target별 설정:

- `@sveltejs/adapter-auto` — deploy 중 Vercel/Netlify/etc 자동 선택
- `@sveltejs/adapter-node` — Node 서버; reverse proxy / Docker
- `@sveltejs/adapter-static` — 순수 정적 사이트 (SSG)
- `@sveltejs/adapter-vercel` — Vercel 특화 기능 (ISR, edge)
- `@sveltejs/adapter-cloudflare` — Cloudflare Pages / Workers
- `@sveltejs/adapter-netlify` — Netlify Functions / Edge

대부분의 앱은 `auto`로 시작한다. target의 기능 (edge runtime, ISR, image opt)이 중요할 때만 명시적 adapter로 전환한다.

## Common mistakes

- **load에서 `fetch` destructure 없이 fetch** — 글로벌 `fetch`가 쿠키를 운반하지 않고 상대 URL을 해석할 수 없으므로 SSR이 깨진다.
- **`load`의 long-running 작업** — `load`는 모든 내비게이션마다 실행된다. 캐시하고, `depends` + `invalidate`을 사용하거나, 백그라운드 job으로 옮긴다.
- **`load` 내부에서 `locals` mutate** — `locals`는 `handle`에서 요청당 한 번 설정된다; `load`/`action`에서는 read-only로 취급한다.
- **새 코드에서 `$app/stores` 사용** — v2는 `$app/state`를 선호한다.
- **form 제출에 `+server.ts` 사용** — progressive enhancement를 잃는다. form action을 사용한다.
- **서버 `load`에서 직렬화 불가능한 데이터 반환** — class, function, proxy가 조용히 drop된다. plain 객체를 반환한다.
