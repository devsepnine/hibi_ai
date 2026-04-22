# SvelteKit 2

SvelteKit is Svelte's official full-stack framework. This reference
covers what you need for real apps: routing, load functions, form
actions, hooks, environment variables, and the `$app/state` APIs
that replaced stores in v2.

## File-system routing

Routes live under `src/routes/`. Each folder is a URL segment.
Special filenames control behavior:

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
- `[[optional]]` — optional: `/blog` or `/blog/hello`
- `[...rest]` — catch-all: `/docs/a/b/c`
- `[param=matcher]` — validated: see `params.*.ts` matchers

### Route groups `(name)`

Folders wrapped in parentheses don't become URL segments. Use them to
give a subset of routes their own layout without changing the URL.

## Load functions — universal vs server

### Universal — `+page.ts`, `+layout.ts`

Runs on the server for the initial render, then in the browser for
client-side nav. Use when the data source is a public endpoint.

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

Key points:

- Use the `fetch` argument, **not** the global — it preserves cookies,
  resolves relative URLs, and on the server avoids the network hop for
  internal routes.
- Throwing `error(...)` triggers the nearest `+error.svelte`.

### Server — `+page.server.ts`, `+layout.server.ts`

Runs only on the server. Required for DB access, private env vars,
filesystem reads, auth checks.

```ts
// +page.server.ts
import type { PageServerLoad } from './$types'

export const load: PageServerLoad = async ({ locals, params }) => {
  const post = await locals.db.posts.findUnique({ where: { slug: params.slug } })
  if (!post) error(404, 'Not found')
  return { post }           // ships to the client as serialized JSON
}
```

Rules:

- Never import `$env/static/private` or `$env/dynamic/private` in a
  universal (`.ts` not `.server.ts`) file — the compiler blocks it.
- The returned object is serialized. Functions, class instances, and
  proxies are stripped. Return plain data.
- Use `locals` (populated in `handle` hook) for request-scoped
  services like DB connections.

### Consuming load data

Every `+page.svelte` / `+layout.svelte` receives the merged load
output as `data`:

```svelte
<script lang="ts">
  import type { PageData } from './$types'
  let { data }: { data: PageData } = $props()
</script>

<h1>{data.post.title}</h1>
```

`PageData` is generated from your `load` return types — no manual
typing.

### Invalidation

When data needs to refresh without a navigation:

```ts
import { invalidate, invalidateAll } from '$app/navigation'
await invalidate('app:posts')        // rerun loads that opted in via `depends('app:posts')`
await invalidateAll()                 // rerun every load on the current route
```

Inside a load:

```ts
export const load: PageLoad = async ({ depends, fetch }) => {
  depends('app:posts')
  return { posts: await (await fetch('/api/posts')).json() }
}
```

## Form actions — the default mutation path

For any user-driven change (create/update/delete), start with form
actions. They work without JS, integrate with `use:enhance` for
progressive enhancement, and keep the typing tight.

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

- `fail(status, data)` — validation failed; return data back to the
  form so the UI can re-render with errors. Stays on the same page.
- `error(status, message)` — unrecoverable; renders `+error.svelte`.
- `redirect(303, path)` — successful mutation; send to a new URL
  (often the created/updated resource).

### Custom enhance behavior

```svelte
<form method="POST" use:enhance={() => async ({ update, result }) => {
  // Run before SvelteKit applies the result
  if (result.type === 'success') confetti()
  await update()          // apply the result (this is what default enhance does)
}}>
```

Use custom enhance when you need side effects on success/failure
(toast, confetti, optimistic UI). Default enhance is fine for most
forms.

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

- `event.locals` is the single place to attach request-scoped data.
  Populate in `handle` so both `load` and `+server.ts` can read.
- `sequence(...)` composes multiple handlers.
- `handleError` is the server-side error trap. Return an object; it
  becomes `$app/state`'s `page.error`.

### `hooks.client.ts`

Smaller surface — mostly `handleError` for client-side errors.

```ts
import type { HandleClientError } from '@sveltejs/kit'

export const handleError: HandleClientError = ({ error }) => {
  console.error(error)
  return { message: 'Something went wrong' }
}
```

## Environment variables

Four flavors, picked based on static/dynamic × public/private:

| Module | Safe in client? | Reads change at runtime? |
|--------|:---:|:---:|
| `$env/static/public`  | ✅ | ❌ (baked at build time) |
| `$env/static/private` | ❌ | ❌ |
| `$env/dynamic/public`  | ✅ | ✅ |
| `$env/dynamic/private` | ❌ | ✅ |

```ts
import { PUBLIC_API_URL }    from '$env/static/public'   // e.g. PUBLIC_API_URL in .env
import { env as priv }       from '$env/dynamic/private' // process.env at runtime
```

Rules:

- `PUBLIC_*` prefix is required for anything in `*/public`.
- Importing from `*/private` in a file that ships to the browser is
  a compile error.
- Prefer `static/*` — better tree-shaking and no runtime lookup cost.
  Use `dynamic/*` only when the value actually changes per deploy /
  per request.

## `$app/state` — replaced `$app/stores` in v2

```ts
import { page, navigating, updated } from '$app/state'

// Plain reactive access — no $page prefix
console.log(page.url.pathname, page.params, page.status, page.error)

if (navigating) {
  console.log('going to', navigating.to?.url.pathname)
}
```

If you see `$app/stores` or `$page` in code, it's a v1/v2-early
pattern — migrate. The v2 `$app/state` API is already tracked by
runes, so just read the fields.

## API endpoints — `+server.ts`

For JSON-ish REST endpoints (mobile clients, webhooks, things that
aren't rendering a page):

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

Prefer form actions for page-owned mutations; use `+server.ts` for
cross-client APIs.

## Page options

Per-file exports on a `+page.ts` / `+layout.ts`:

```ts
export const prerender = true       // emit as static HTML at build
export const ssr = false             // SPA: skip server rendering
export const csr = true              // keep client hydration (default)
export const trailingSlash = 'never' // url normalization
```

Use `prerender = true` for genuinely static pages (marketing, docs)
— cuts deploy size and cold-start cost. Combine with `adapter-static`
for full SSG sites.

## Adapters (quick map)

Set per-target via `svelte.config.js`:

- `@sveltejs/adapter-auto` — picks Vercel/Netlify/etc automatically
  during deploys
- `@sveltejs/adapter-node` — Node server; reverse proxy / Docker
- `@sveltejs/adapter-static` — pure static site (SSG)
- `@sveltejs/adapter-vercel` — Vercel-specific features (ISR, edge)
- `@sveltejs/adapter-cloudflare` — Cloudflare Pages / Workers
- `@sveltejs/adapter-netlify` — Netlify Functions / Edge

Most apps start with `auto`. Switch to an explicit adapter only when
the target's features (edge runtime, ISR, image opt) matter.

## Common mistakes

- **Fetch without destructuring `fetch`** in a load — breaks SSR
  because the global `fetch` doesn't carry cookies and can't resolve
  relative URLs.
- **Long-running work in `load`** — `load` runs on every navigation.
  Cache, use `depends` + `invalidate`, or move to a background job.
- **Mutating `locals` inside `load`** — `locals` is set once per
  request in `handle`; treat it as read-only from `load`/`action`.
- **Using `$app/stores`** in new code — v2 prefers `$app/state`.
- **Using `+server.ts` for form submissions** — loses progressive
  enhancement. Use form actions.
- **Returning non-serializable data** from a server `load` — classes,
  functions, and proxies silently drop. Return plain objects.
