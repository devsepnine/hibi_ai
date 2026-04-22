---
name: svelte-5
description: Write Svelte 5 components and SvelteKit 2 apps using the runes-based reactivity model ($state, $derived, $effect, $props, $bindable), callback-prop events (no createEventDispatcher), and the snippet/@render pattern (no slots). Covers SvelteKit 2 routing, load functions, form actions, hooks, environment variables, and the $app/state APIs that replaced stores. Use whenever the user writes, reviews, or migrates Svelte/SvelteKit code; whenever they mention runes, `.svelte.ts` files, `+page.svelte` / `+layout.ts` / `+page.server.ts`, or ask for "a Svelte component," "a SvelteKit route," a v4→v5 migration, or anything involving reactive declarations in Svelte. 스벨트 컴포넌트, SvelteKit 라우팅, runes 마이그레이션, 스벨트 5 작성.
keywords: [svelte, svelte-5, 스벨트, sveltekit, sveltekit-2, 스벨트킷, runes, $state, $derived, $effect, $props, $bindable, snippet, load, form-actions, hooks, migration, 마이그레이션]
---

# Svelte 5

Produce idiomatic Svelte 5 components and SvelteKit 2 routes. Defaults
to **runes mode** (the current Svelte 5 default), the `{@render
children()}` / snippet pattern, and callback-prop events. Never
revert to legacy `export let` / `on:event` / `createEventDispatcher`
unless the user is explicitly maintaining a pre-5 codebase.

Read the matching reference file when working:

- [references/runes.md](references/runes.md) — `$state`, `$derived`,
  `$effect`, `$props`, `$bindable`, `$inspect` with worked examples
  and when each one is the wrong tool
- [references/sveltekit.md](references/sveltekit.md) — routing, `load`
  functions (universal vs server), form actions + `use:enhance`,
  hooks, `$env`, `$app/state`
- [references/migrating-from-svelte-4.md](references/migrating-from-svelte-4.md)
  — mechanical translation of `export let`, `$:`, slots,
  `createEventDispatcher`, `on:event`, `<svelte:component>` into v5

## Defaults

Assume the user is on:

- **Svelte 5** (runes mode, released 2024) — the default since
  `svelte@5`.
- **SvelteKit 2** — `+page.svelte`, `+layout.ts`, `+page.server.ts`
  file-system routing; form actions; `$app/state` (not `$app/stores`).
- **TypeScript** unless the user's file explicitly uses plain JS.

When in doubt, pick the modern path — runes over stores, snippets
over slots, callback props over dispatchers.

## Runes at a glance

| Rune | Role | When to use |
|------|------|-------------|
| `$state(x)` | Reactive value with deep proxying | Any value that drives UI and changes over time |
| `$state.raw(x)` | Shallow — proxy not applied | Large immutable structures where deep proxying is wasteful |
| `$derived(expr)` | Pure computed value, re-runs on dep change | Expressions from other reactive state |
| `$derived.by(() => ...)` | `$derived` that takes a function | Derived values needing a multi-statement body |
| `$effect(() => ...)` | Side effect that re-runs on dep change | DOM, subscriptions, analytics, canvas — **not** state updates |
| `$effect.pre(() => ...)` | Effect before DOM commit | Measurements, layout reads before paint |
| `$props()` | Component props | Every component that receives inputs |
| `$bindable(default?)` | Opt-in two-way binding slot | Form wrappers where parent wants `bind:value` |
| `$inspect(...)` | Dev-only log on change | Debugging reactivity |

Details, traps, and examples: [runes.md](references/runes.md).

## Component shape

A Svelte 5 component is a `.svelte` file with three ordered slots:

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

Key choices baked in:

- **Props are `$props()`**, not `export let`. Destructured, typed.
- **Children are `Snippet`**, rendered with `{@render children?.()}` —
  not `<slot>`.
- **Events are attributes** (`onclick`, `oninput`) — not
  `on:click`. No `createEventDispatcher`; emit via callback props
  (`onclose`, `onsubmit`, etc.).
- **Transitions unchanged** — `svelte/transition` and `svelte/motion`
  still the right imports.

Full copy-ready template: [assets/component.svelte](assets/component.svelte).

## Sharing reactive state (`.svelte.ts`)

Runes work inside `.svelte`, `.svelte.js`, and `.svelte.ts` files.
Use a `.svelte.ts` module when multiple components need the same
reactive source. **Don't use legacy `writable`/`readable` stores for
new code** — they still work for backwards compatibility only.

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

Then in any component: `import { cart } from './cart.svelte.ts'`.
`cart.items` and `cart.total` stay reactive across component
boundaries. Full pattern: [assets/store.svelte.ts](assets/store.svelte.ts).

## SvelteKit 2 — file roles

| File | Runs on | Purpose |
|------|---------|---------|
| `+page.svelte` | Client (hydrated from SSR) | The page UI |
| `+page.ts` | Universal (server + client) | `load`; safe to use in either env |
| `+page.server.ts` | **Server only** | `load`, `actions`, access to secrets / DB |
| `+layout.svelte` | Client | Shared wrapping UI (nav, sidebar) |
| `+layout.ts` / `+layout.server.ts` | ibid | Shared data for child routes |
| `+error.svelte` | Client | Fallback UI when `load` throws |
| `+server.ts` | Server | REST-style API endpoint (`GET`, `POST`, ...) |
| `hooks.server.ts` | Server | `handle`, `handleFetch`, `handleError` |
| `hooks.client.ts` | Client | `handleError` client-side |

Full routing + load + actions flow: [sveltekit.md](references/sveltekit.md).

## Load functions — universal vs server

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

Guidelines:

- Use `+page.ts` if the endpoint is public and you want client-side
  nav to skip a server round-trip.
- Use `+page.server.ts` when the data source is server-only (DB,
  secrets, filesystem).
- Never import `$env/static/private` or `$env/dynamic/private` from a
  universal file — the compiler blocks it, for good reason.

## Form actions — the first tool for mutations

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

`use:enhance` gives progressive enhancement (the form works with JS
disabled) plus fine-grained control over the response. Prefer form
actions over `fetch('/api/...', { method: 'POST' })` for mutations
that belong to a page.

## Events — callback props, not dispatchers

```svelte
<!-- Modal.svelte -->
<script lang="ts">
  let { onclose }: { onclose: () => void } = $props()
</script>

<button onclick={onclose}>Close</button>

<!-- usage -->
<Modal onclose={() => (modalOpen = false)} />
```

No `createEventDispatcher`. The parent passes a function; the child
calls it. This is cheaper at runtime, fully typed, and works with any
callback shape (single arg, multi arg, returning promises).

## Snippets — the slot replacement

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

Snippets accept positional arguments (unlike slots), are typed via
`Snippet<[Args]>`, and can be passed as props anywhere. Use them for
anything that used to be a slot with props.

## Migrating from Svelte 4

If the user has v4 code, mechanical translations:

| v4 | v5 |
|----|----|
| `export let name` | `let { name } = $props()` |
| `let count = 0`<br>`$: doubled = count * 2` | `let count = $state(0)`<br>`const doubled = $derived(count * 2)` |
| `$: if (x > 10) warn()` | `$effect(() => { if (x > 10) warn() })` |
| `on:click={handler}` | `onclick={handler}` |
| `<slot />` | `{@render children?.()}` + `children?: Snippet` in props |
| `<slot name="foo" />` | named-prop snippet: `foo?: Snippet` + `{@render foo?.()}` |
| `createEventDispatcher()` + `dispatch('save')` | `onsave` callback prop |
| `<svelte:component this={Comp} />` | `<Comp />` (components are values in v5) |
| `writable(x)` in `.ts` | `let v = $state(x)` in `.svelte.ts` |

Detailed transition pitfalls (reactive loops, effect timing, legacy
mode): [migrating-from-svelte-4.md](references/migrating-from-svelte-4.md).

## Anti-patterns

- **Don't write to reactive state inside `$effect`** — causes
  re-run loops. Put the update in the event handler that triggered it.
  If genuine cascading is needed, use `$derived` (which is pure).
- **Don't reach for `svelte/store` in new code** — `.svelte.ts` +
  `$state` covers cross-component sharing without the
  subscribe/unsubscribe dance.
- **Don't use `$app/stores`** in SvelteKit 2 code — it's deprecated in
  favor of `$app/state`. `page.url`, `page.params`, `page.status` are
  now plain reactive properties, no `$page` prefix.
- **Don't mix `export let` with `$props()`** — the compiler will
  warn, and the component is locked out of runes features for that
  variable.
- **Don't set DOM properties imperatively when a reactive attribute
  would do** — `$effect` that calls `element.classList.add(...)` is
  usually a `class={{ ... }}` shorthand waiting to happen.

## Integration notes

- Tests — use `vitest` with `@testing-library/svelte` v5+ (snippets
  support). For SvelteKit, `playwright` covers e2e.
- Styling — scoped `<style>` by default; `:global(...)` for escape.
  Tailwind / UnoCSS integrate via `sv add tailwindcss`.
- Type generation — SvelteKit auto-generates `./$types` per route
  (`PageLoad`, `PageServerLoad`, `Actions`, `PageData`).
