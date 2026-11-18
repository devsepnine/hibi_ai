---
name: svelte-5
description: Svelte 5 + SvelteKit 2 with runes `$state`/`$derived`/`$effect`, snippets over slots, callback events, load functions, form actions. Use when writing Svelte components. 스벨트 5, SvelteKit 라우팅, 컴포넌트, runes 마이그레이션.
---

# Svelte 5

Check the installed Svelte version, compiler options, and component syntax. Svelte 5 supports legacy and runes components in the same project. For a targeted fix, preserve the component’s current mode, public props, slots, and events. Migrate when the task or project calls for it. For a new component, use runes where the project leaves the mode open.

Read the matching reference file when working:

- [references/runes.md](references/runes.md): `$state`, `$derived`,
  `$effect`, `$props`, `$bindable`, `$inspect` with worked examples
  and when each one is the wrong tool
- [references/sveltekit.md](references/sveltekit.md): routing, `load`
  functions split into universal and server, form actions + `use:enhance`,
  hooks, `$env`, `$app/state`
- [references/migrating-from-svelte-4.md](references/migrating-from-svelte-4.md):
  mechanical translation of `export let`, `$:`, slots,
  `createEventDispatcher`, `on:event`, `<svelte:component>` into v5

## Defaults for new components

The following examples use Svelte 5 runes, TypeScript, snippets and callback props. Use the language and conventions of the target project. SvelteKit examples use version 2 routing and form actions. Reactive reads from `$app/state` require runes and SvelteKit 2.12 or later.

## Runes at a glance

| Rune | Role | When to use |
|------|------|-------------|
| `$state(x)` | Reactive value with deep proxying | Any value that drives UI and changes over time |
| `$state.raw(x)` | Shallow, proxy not applied | Large immutable structures where deep proxying is wasteful |
| `$derived(expr)` | Pure computed value, re-runs on dep change | Expressions from other reactive state |
| `$derived.by(() => ...)` | `$derived` that takes a function | Derived values needing a multi-statement body |
| `$effect(() => ...)` | Side effect that re-runs on dep change | DOM, subscriptions, analytics, canvas, but **not** state updates |
| `$effect.pre(() => ...)` | Effect before DOM commit | Measurements, layout reads before paint |
| `$props()` | Component props | Every component that receives inputs |
| `$bindable(default?)` | Opt-in two-way binding slot | Form wrappers where parent wants `bind:value` |
| `$inspect(...)` | Dev-only log on change | Debugging reactivity |

Details, traps, and examples: [runes.md](references/runes.md).

## Component shape

This runes component uses a script, markup, and scoped styles.

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
- **Children are `Snippet`**, rendered with `{@render children?.()}`,
  not `<slot>`.
- **Events are attributes** such as `onclick` and `oninput`, not
  `on:click`. No `createEventDispatcher`; emit via callback props
  such as `onclose` and `onsubmit`.
- **Transitions unchanged**: `svelte/transition` and `svelte/motion`
  still the right imports.

Full copy-ready template: [assets/component.svelte](assets/component.svelte).

## Sharing reactive state in `.svelte.ts`

For new shared reactive state, a runes module can live in a `.svelte.ts` or `.svelte.js` file. Keep a store where callers depend on its subscription contract.

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

## SvelteKit 2: file roles

| File | Runs on | Purpose |
|------|---------|---------|
| `+page.svelte` | Client, hydrated from SSR | The page UI |
| `+page.ts` | Universal, server and client | `load`; safe to use in either env |
| `+page.server.ts` | **Server only** | `load`, `actions`, access to secrets / DB |
| `+layout.svelte` | Client | Shared wrapping UI such as nav and sidebar |
| `+layout.ts` / `+layout.server.ts` | ibid | Shared data for child routes |
| `+error.svelte` | Client | Fallback UI when `load` throws |
| `+server.ts` | Server | REST-style API endpoint for `GET`, `POST`, ... |
| `hooks.server.ts` | Server | `handle`, `handleFetch`, `handleError` |
| `hooks.client.ts` | Client | `handleError` client-side |

Full routing + load + actions flow: [sveltekit.md](references/sveltekit.md).

## Load functions: universal vs server

```ts
// +page.ts  (universal — runs on server first, then client on nav)
import { error } from '@sveltejs/kit'
import type { PageLoad } from './$types'
export const load: PageLoad = async ({ fetch, params }) => {
  const res = await fetch(`/api/posts/${params.slug}`)   // use the scoped fetch
  if (res.status === 404) error(404, 'Post not found')
  if (!res.ok) error(502, 'Unable to load post')
  return { post: await res.json() }
}
```

```ts
// +page.server.ts  (server-only — access secrets, DB)
import { error } from '@sveltejs/kit'
import type { PageServerLoad } from './$types'
export const load: PageServerLoad = async ({ locals, params }) => {
  const post = await locals.db.posts.findUnique({
    where: { slug: params.slug },
    select: { slug: true, title: true }
  })
  if (!post) error(404, 'Post not found')
  return { post: { slug: post.slug, title: post.title } }
}
```

Guidelines:

- Server load results travel to the browser. Return only the fields the user may read.
- Use `+page.ts` if the endpoint is public and you want client-side
  nav to skip a server round-trip.
- Use `+page.server.ts` when the data source is server-only, such as a DB,
  secrets, or the filesystem.
- Never import `$env/static/private` or `$env/dynamic/private` from a
  universal file. The compiler blocks it, for good reason.

## Form actions: the first tool for mutations

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

`use:enhance` gives progressive enhancement, meaning the form works with JS
disabled, plus fine-grained control over the response. Prefer form
actions over `fetch('/api/...', { method: 'POST' })` for mutations
that belong to a page.

## Callback props in runes components

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
callback shape: single arg, multi arg, or returning promises.

## Snippets: the slot replacement

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

Unlike slots, snippets accept positional arguments, are typed via
`Snippet<[Args]>`, and can be passed as props anywhere. Use them for
anything that used to be a slot with props.

## Migrating from Svelte 4

Use these translations when the task includes migrating the component to runes.

| v4 | v5 |
|----|----|
| `export let name` | `let { name } = $props()` |
| `let count = 0`<br>`$: doubled = count * 2` | `let count = $state(0)`<br>`const doubled = $derived(count * 2)` |
| `$: if (x > 10) warn()` | `$effect(() => { if (x > 10) warn() })` |
| `on:click={handler}` | `onclick={handler}` |
| `<slot />` | `{@render children?.()}` + `children?: Snippet` in props |
| `<slot name="foo" />` | named-prop snippet: `foo?: Snippet` + `{@render foo?.()}` |
| `createEventDispatcher()` + `dispatch('save')` | `onsave` callback prop |
| `<svelte:component this={Comp} />` | `<Comp />`, since components are values in v5 |
| `writable(x)` in `.ts` | `let v = $state(x)` in `.svelte.ts` |

Detailed transition pitfalls, covering reactive loops, effect timing, and legacy
mode: [migrating-from-svelte-4.md](references/migrating-from-svelte-4.md).

## Anti-patterns

- **Don't write to reactive state inside `$effect`**. It causes
  re-run loops. Put the update in the event handler that triggered it.
  If genuine cascading is needed, use `$derived`, which is pure.
- Keep `$app/stores` subscription behavior in legacy components. In a migration, switch route reads to `$app/state` together with the runes that track them.
- A component using `$props()` cannot also declare props with `export let`. The compiler rejects this combination.
- **Don't set DOM properties imperatively when a reactive attribute
  would do**. A `$effect` that calls `element.classList.add(...)` is
  usually a `class={{ ... }}` shorthand waiting to happen.

## Integration notes

- Tests: use `vitest` with `@testing-library/svelte` v5+, which has snippets
  support. For SvelteKit, `playwright` covers e2e.
- Styling: scoped `<style>` by default; `:global(...)` for escape.
  Tailwind / UnoCSS integrate via `sv add tailwindcss`.
- Type generation: SvelteKit auto-generates `./$types` per route, such as
  `PageLoad`, `PageServerLoad`, `Actions`, and `PageData`.
